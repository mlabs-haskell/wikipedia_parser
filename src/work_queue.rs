use rayon::iter::{ParallelBridge, ParallelIterator};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::mpsc;
use std::thread::JoinHandle;

const QUEUE_SIZE: usize = 1024;

const K: usize = 1024;
const M: usize = 1024 * K;
const G: usize = 1024 * M;
const OUTPUT_BUFFER_SIZE: usize = 2 * G;
const INDEX_BUFFER_SIZE: usize = 1 * G;

pub struct WorkQueue {
    parser_sender: Option<mpsc::SyncSender<(String, Vec<u8>)>>,
    parser_thread: JoinHandle<()>,
    writer_thread: JoinHandle<()>,
}

impl WorkQueue {
    pub fn new<F>(data_file: String, index_file: String, text_processor: F) -> Self
    where
        F: Fn(&[u8], &str) -> String + Sync + Send + 'static,
    {
        let (writer_sender, writer_receiver) = mpsc::sync_channel::<(String, String)>(QUEUE_SIZE);
        let (parser_sender, parser_receiver) = mpsc::sync_channel::<(String, Vec<u8>)>(QUEUE_SIZE);

        // Start the writer thread
        let writer_thread =
            std::thread::spawn(move || file_writer(data_file, index_file, writer_receiver));

        // Iterate over the elements in the parser channel parallely, and run text_processor in a
        // thread pool. Send the result over to the writer thread.
        let parser_thread = std::thread::spawn(move || {
            parser_receiver.into_iter().par_bridge().for_each_with(
                writer_sender,
                |writer_sender, (title, contents)| {
                    // Process the text
                    let text = (text_processor)(&contents, &title);

                    // Send the output to the writer thread
                    writer_sender.send((title, text)).unwrap();
                },
            )
        });

        Self {
            parser_sender: Some(parser_sender),
            parser_thread,
            writer_thread,
        }
    }

    pub fn queue(&mut self, text: Vec<u8>, title: String) {
        self.parser_sender
            .as_ref()
            .unwrap()
            .send((title, text))
            .unwrap();
    }

    pub fn wait_for_completion(mut self) {
        drop(self.parser_sender.take());
        self.parser_thread.join().unwrap();
        self.writer_thread.join().unwrap();
    }
}

fn file_writer(data_file: String, index_file: String, rx: mpsc::Receiver<(String, String)>) {
    let data_file = File::create(data_file).unwrap();
    let mut data_file_writer = BufWriter::with_capacity(OUTPUT_BUFFER_SIZE, data_file);

    let index_file = File::create(index_file).unwrap();
    let mut index_file_writer = BufWriter::with_capacity(INDEX_BUFFER_SIZE, index_file);

    let mut pos = 0;

    loop {
        let (name, contents) = match rx.recv() {
            Err(_) => break,
            Ok(x) => x,
        };

        let bytes = contents.as_bytes();
        let bytes_written = data_file_writer.write(bytes).unwrap();

        // This should be the case. Just in case this assumption is wrong, do an early exit.
        assert!(bytes_written == bytes.len());

        let written = index_file_writer
            .write(format!("{}: {}\n", pos, name).as_bytes())
            .unwrap();
        assert!(written != 0);

        pos += bytes_written;
    }

    data_file_writer.flush().unwrap();
    index_file_writer.flush().unwrap();
}
