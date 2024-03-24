//! Parallel File Reader to saturate NVMe read queues
//! See [ParFile](self::ParFile)

use std::{
    fs::File,
    io::{Read, Seek},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, RecvError, SendError, SyncSender},
        Arc,
    },
    thread,
};

use thiserror::Error;

#[cfg(test)]
mod tests;

/// Parallel File Reader to saturate NVMe read queues
/// Spawns N threads which read 0..buf_size, buf_size..2*buf_size, .., slices of the file in
/// parallel using multiple syscalls, and send these through a Channel to the main thread.
/// The main thread reads the buffers sent by each of these threads in sequence during the
/// `Read::read()` call.
/// Once a buffer is read it is returned back to the thread for the next round.
pub struct ParFile {
    // The thread from which we are reading
    current_thread: usize,
    // If we have received a buffer from a thread but the user hasn't read it to the end, store it
    // here
    current_buffer: Option<Vec<u8>>,
    // The offset from the start of the current buffer, to serve the next read() from
    current_buffer_offset: usize,
    // Handles to active threads
    threads: Vec<ThreadHandle>,
    active_thread_count: Arc<AtomicU64>,
}

#[derive(Debug, Error)]
pub enum ParFileError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Recv Error: {0}")]
    RecvError(#[from] RecvError),
    #[error("Send Error: {0}")]
    SendError(#[from] SendError<Vec<u8>>),
}

impl ParFile {
    pub fn new(filename: String, block_size: u64, queue_size: u64, num_threads: u64) -> Self {
        let active_thread_count = Arc::new(AtomicU64::new(0));

        let mut threads = Vec::new();

        for thread_idx in 0..num_threads {
            let thread = ThreadHandle::new_spawn(
                filename.clone(),
                queue_size,
                block_size,
                active_thread_count.clone(),
                thread_idx,
                num_threads,
            );
            threads.push(thread);
        }

        Self {
            current_thread: 0,
            current_buffer: None,
            current_buffer_offset: 0,
            threads,
            active_thread_count,
        }
    }

    pub fn active_thread_count(&self) -> Arc<AtomicU64> {
        self.active_thread_count.clone()
    }
}

impl Read for ParFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // If we don't have a partially read buffer
        // - request a new one from the current thread
        if self.current_buffer.is_none() {
            let thread = &mut self.threads[self.current_thread];
            let receiver = &mut thread.receiver;
            let new_buffer = receiver.recv().ok();
            if new_buffer.is_none() {
                // The sender thread exited.
                // This could mean:
                //  - The sender thread reached EOF
                //    return 0 to indicate EOF
                //  - The sender threads exited
                //    ie, ThreadHandle is dropped or the program is exiting.
                //    We don't care about the return value in that case.
                return Ok(0);
            }
            self.current_buffer = Some(new_buffer.unwrap());
        }

        let current_buffer = self.current_buffer.as_mut().unwrap();

        let current_buffer = &current_buffer[self.current_buffer_offset..];
        let n = buf.len().min(current_buffer.len());

        buf[..n].copy_from_slice(&current_buffer[..n]);

        // If the buffer is read to the end:
        // - increment current thread,
        // - unset current buffer
        if n == current_buffer.len() {
            let thread = &mut self.threads[self.current_thread];
            let sender = &mut thread.return_sender;
            let _ = sender.send(self.current_buffer.take().unwrap());
            self.current_buffer_offset = 0;
            self.current_thread += 1;
            if self.current_thread >= self.threads.len() {
                self.current_thread = 0;
            }
        } else {
            self.current_buffer_offset += n;
        }

        Ok(n)
    }
}

struct ThreadHandle {
    // Receive buffer to be filled here
    receiver: Receiver<Vec<u8>>,
    // Send filled buffer through here
    return_sender: SyncSender<Vec<u8>>,
}

impl ThreadHandle {
    fn new_spawn(
        filename: String,
        queue_size: u64,
        block_size: u64,
        active_thread_count: Arc<AtomicU64>,
        thread_idx: u64,
        num_threads: u64,
    ) -> Self {
        // Channel for buffers filled with data read from file
        let (sender, receiver) = mpsc::sync_channel::<Vec<u8>>(queue_size as _);
        // Return channel for buffers completely read by the user
        let (return_sender, return_receiver) = mpsc::sync_channel::<Vec<u8>>(queue_size as _);

        // Fill return_sender with buffers of size block_size
        for _ in 0..queue_size {
            return_sender.send(vec![0; block_size as _]).unwrap();
        }

        let active_thread_count = active_thread_count.clone();

        thread::spawn(move || {
            active_thread_count.fetch_add(1, Ordering::Relaxed);

            let mut file = File::open(filename)?;
            file.seek(std::io::SeekFrom::Start(thread_idx * block_size))?;

            let reader = Reader {
                file,
                stride: (block_size * num_threads) as _,
                sender,
                return_receiver,
            };

            let result = reader.run();

            active_thread_count.fetch_sub(1, Ordering::Relaxed);

            result
        });

        ThreadHandle {
            receiver,
            return_sender,
        }
    }
}

struct Reader {
    file: File,
    stride: usize,
    sender: SyncSender<Vec<u8>>,
    return_receiver: Receiver<Vec<u8>>,
}

impl Reader {
    fn run(mut self) -> Result<(), ParFileError> {
        loop {
            // Get a new buffer from the return channel
            let mut buf = self.return_receiver.recv()?;
            // Read into the buffer
            let n = self.file.read(&mut buf)?;
            if n != buf.len() {
                // If buffer is not fully read, it must be the end of stream
                // Not sure if this assertion holds universally.
                // So panic if it breaks and fix it later.
                assert!(
                    self.file.stream_position()? >= self.file.metadata()?.len(),
                    "n: {}; buf.len(): {}",
                    n,
                    buf.len(),
                );
                // Resize the buffer to the length returned by read()
                // so that the receiver gets the correct length
                buf.resize(n, 0);
                self.sender.send(buf)?;
                break;
            }
            self.sender.send(buf)?;
            // Seek to i * stride
            //  where i: iteration index
            self.file
                .seek(std::io::SeekFrom::Current((self.stride - n) as i64))?;
        }
        Ok(())
    }
}
