use rayon::iter::{ParallelBridge, ParallelIterator};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::mpsc;

const QUEUE_SIZE: usize = 1024;
const OUTPUT_FILENAME: &'static str = "output.jsonl";
const INDEX_FILENAME: &'static str = "output-index.txt";

const K: usize = 1024;
const M: usize = 1024 * K;
const G: usize = 1024 * M;
const OUTPUT_BUFFER_SIZE: usize = 2 * G;
const INDEX_BUFFER_SIZE: usize = 1 * G;

pub struct WorkQueue {
    parser_sender: mpsc::SyncSender<(String, Vec<u8>)>,
}

impl WorkQueue {
    pub fn new<F>(root_dir: String, text_processor: F) -> Self
    where
        F: Fn(&[u8], &str) -> String + Sync + Send + 'static,
    {
        let (writer_sender, writer_receiver) = mpsc::sync_channel::<(String, String)>(QUEUE_SIZE);
        let (parser_sender, parser_receiver) = mpsc::sync_channel::<(String, Vec<u8>)>(QUEUE_SIZE);

        // Start the writer thread
        std::thread::spawn(move || file_writer(root_dir, writer_receiver));

        // Iterate over the elements in the parser channel parallely, and run text_processor in a
        // thread pool. Send the result over to the writer thread.
        std::thread::spawn(move || {
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

        Self { parser_sender }
    }

    pub fn queue(&mut self, text: Vec<u8>, title: String) {
        self.parser_sender.send((title, text)).unwrap();
    }
}

fn file_writer(root_dir: String, rx: mpsc::Receiver<(String, String)>) {
    std::fs::create_dir_all(&root_dir).unwrap();

    let file = File::create(format!("{}/{}", root_dir, OUTPUT_FILENAME)).unwrap();
    let mut file_writer = BufWriter::with_capacity(OUTPUT_BUFFER_SIZE, file);

    let index_file = File::create(format!("{}/{}", root_dir, INDEX_FILENAME)).unwrap();
    let mut index_file_writer = BufWriter::with_capacity(INDEX_BUFFER_SIZE, index_file);

    let mut pos = 0;

    loop {
        let (name, contents) = match rx.recv() {
            Err(_) => break,
            Ok(x) => x,
        };

        let bytes = contents.as_bytes();
        let bytes_written = file_writer.write(bytes).unwrap();

        // This should be the case. Just in case this assumption is wrong, do an early exit.
        assert!(bytes_written == bytes.len());

        index_file_writer
            .write(format!("{}: {}\n", pos, name).as_bytes())
            .unwrap();

        pos += bytes_written;
    }

    file_writer.flush().unwrap();
}
