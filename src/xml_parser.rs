use core::panic;
use std::collections::BTreeSet;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::path::Path;
use std::str;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use quick_xml::Error;
use quick_xml::Result;

use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::tree::Tree;

const NUM_THREADS: usize = 64;

pub struct XMLParser<F: Fn(&str) -> String + Clone + Sync + Send + Copy + 'static> {
    text_processor: F,
    reader: Reader<BufReader<File>>,
    num_articles: usize,
    root_dir: String,
    file_size: u64, // for tracking progress
    work_queue: WorkQueue,
}

impl<F: Fn(&str) -> String + Clone + Sync + Send + Copy> XMLParser<F> {
    pub fn new(root_dir: String, text_processor: F, filename: &str) -> Result<Self> {
        let file_size = std::fs::File::open(filename)?.metadata()?.len();
        let reader = Reader::from_file(filename)?;
        let work_queue = WorkQueue::new();
        Ok(Self {
            text_processor,
            reader,
            num_articles: 0,
            root_dir,
            file_size,
            work_queue,
        })
    }

    // Main XML parsing function
    pub fn parse_xml(&mut self) -> Result<()> {
        let mut buffer = Vec::new();
        match self.reader.read_event_into(&mut buffer) {
            Err(e) => self.terminate(e),
            Ok(Event::Start(e)) => {
                if e.name().into_inner() == b"mediawiki" {
                    self.parse_mediawiki()
                } else {
                    Err(Error::TextNotFound)
                }
            }
            _ => Err(Error::TextNotFound),
        }
    }

    // Parse the body of the XML page
    fn parse_mediawiki(&mut self) -> Result<()> {
        let mut buffer = Vec::new();
        let file_size = self.file_size;
        let start_time = SystemTime::now();
        loop {
            let pos = self.reader.buffer_position();
            let pct = 100.0 * (pos as f64) / (file_size as f64);

            let elapsed = start_time.elapsed().unwrap();
            let rate = pos as f64 / elapsed.as_secs_f64();
            let rate_mb = rate / 1024.0 / 1024.0;
            let eta_secs = file_size as f64 / rate;
            let eta_mins = eta_secs / 60.0;

            print!(
                "Progress: {:.2}% {}/{} | {:.2} MB/sec {:.2} mins ETA \r",
                pct, pos, file_size, rate_mb, eta_mins
            );

            match self.reader.read_event_into(&mut buffer) {
                Err(e) => self.terminate(e),
                Ok(Event::Start(e)) => {
                    let tag = e.name().into_inner();
                    match tag {
                        b"page" => self.parse_page()?,
                        b"siteinfo" => {
                            let mut garbage = Vec::new();
                            self.reader.read_to_end_into(QName(tag), &mut garbage)?;
                        }
                        _ => {
                            println!("Unknown tag: {}", String::from_utf8_lossy(tag));
                            let mut garbage = Vec::new();
                            self.reader.read_to_end_into(QName(tag), &mut garbage)?;
                        }
                    }
                }
                Ok(Event::Eof) => break,
                _ => (),
            }
        }

        Ok(())
    }

    fn parse_page(&mut self) -> Result<()> {
        let mut buffer = Vec::new();
        let mut title = Vec::new();
        let mut text = Vec::new();

        // Parse the page
        loop {
            match self.reader.read_event_into(&mut buffer) {
                Err(e) => self.terminate(e),
                Ok(Event::Empty(e)) => {
                    let tag = e.name().into_inner();
                    if tag == b"redirect" {
                        // We don't care about redirect pages
                        let mut garbage = Vec::new();
                        self.reader.read_to_end_into(QName(b"page"), &mut garbage)?;
                        return Ok(());
                    }
                }
                Ok(Event::Start(e)) => {
                    let tag = e.name().into_inner();
                    match tag {
                        b"title" => match self.reader.read_event_into(&mut title) {
                            Err(e) => self.terminate(e),
                            Ok(Event::Text(_)) => (),
                            _ => return Err(Error::TextNotFound),
                        },
                        b"ns" | b"id" => {
                            let mut garbage = Vec::new();
                            self.reader.read_to_end_into(QName(tag), &mut garbage)?;
                        }
                        b"revision" => self.parse_revision(&mut text)?,
                        _ => {
                            println!("Unknown tag: {}", String::from_utf8_lossy(tag));
                            let mut garbage = Vec::new();
                            self.reader.read_to_end_into(QName(tag), &mut garbage)?;
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    if e.name().into_inner() == b"page" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                _ => (),
            }
        }

        // Skip technical articles about Wikipedia itself
        let title = String::from_utf8(title)?;
        if title.starts_with("Wikipedia:")
            || title.starts_with("Portal:")
            || title.starts_with("File:")
            || title.starts_with("Template:")
            || title.starts_with("Category:")
            || title.starts_with("Draft:")
            || title.starts_with("Module:")
            || title.starts_with("MediaWiki:")
            || title.starts_with("Help:")
            || title.to_lowercase().ends_with("(disambiguation)")
        {
            return Ok(());
        }

        let article_id = self.num_articles;
        self.num_articles += 1;
        self.work_queue.queue(
            article_id,
            String::from_utf8_lossy(&text).into_owned(),
            title,
            self.text_processor.clone(),
            self.root_dir.clone(),
        );

        Ok(())
    }

    fn parse_revision(&mut self, text: &mut Vec<u8>) -> Result<()> {
        let mut buffer = Vec::new();
        loop {
            match self.reader.read_event_into(&mut buffer) {
                Err(e) => self.terminate(e),
                Ok(Event::Start(e)) => {
                    let tag = e.name().into_inner();
                    match tag {
                        b"id" | b"parentid" | b"timestamp" | b"contributor" | b"minor"
                        | b"comment" | b"model" | b"format" | b"sha1" => {
                            let mut garbage = Vec::new();
                            self.reader.read_to_end_into(QName(tag), &mut garbage)?;
                        }
                        b"text" => {
                            let mut text_buffer = Vec::new();
                            match self.reader.read_event_into(&mut text_buffer) {
                                Err(e) => self.terminate(e),
                                Ok(Event::Text(e)) => {
                                    *text = e.to_vec();
                                }
                                _ => return Err(Error::TextNotFound),
                            }
                        }
                        _ => {
                            println!("Unknown tag: {}", String::from_utf8_lossy(tag));
                            let mut garbage = Vec::new();
                            self.reader.read_to_end_into(QName(tag), &mut garbage)?;
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    if e.name().into_inner() == b"revision" {
                        break;
                    }
                }
                Ok(Event::Eof) => return Err(Error::TextNotFound),
                _ => (),
            }
        }

        Ok(())
    }

    // Universal error
    fn terminate(&self, e: Error) -> ! {
        panic!(
            "Error at position {}: {:?}",
            self.reader.buffer_position(),
            e
        )
    }
}

fn write_file(root_dir: &str, title: &str, text: &str, article_id: usize) -> Result<()> {
    // Figure out where to write the file
    let filename = format!("{}_{}", article_id, title);
    let filename = filename.replace(|c: char| !c.is_alphanumeric(), "_");
    let filename: String = filename.chars().take(100).collect();
    let mut sub_dir: String = filename.chars().take(4).collect();
    if sub_dir.ends_with(|c: char| !c.is_numeric()) {
        sub_dir = "0000".to_string();
    }
    let path = format!("{}/{}/{}.json", root_dir, &sub_dir, &filename);

    // Create the directories that will contain the file
    let path = Path::new(&path);
    let prefix = path.parent().unwrap();
    create_dir_all(prefix)?;

    // Convert text to JSON
    let tree = Tree::from_string(title, text);
    let text = serde_json::to_string_pretty(&tree).unwrap();

    // Write the file
    let mut file = File::create(path);
    if let Ok(f) = file.as_mut() {
        f.write_all(text.as_bytes())?;
    } else {
        panic!("Could not write file: {}", filename);
    }

    Ok(())
}

struct WorkQueue {
    thread_pool: ThreadPool,
    processing_articles: Arc<Mutex<BTreeSet<usize>>>,
    active_threads: Arc<AtomicUsize>,
}

impl WorkQueue {
    fn new() -> Self {
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build()
            .unwrap();
        Self {
            thread_pool,
            processing_articles: Arc::new(Mutex::new(BTreeSet::new())),
            active_threads: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn queue<F>(
        &mut self,
        article_id: usize,
        text: String,
        title: String,
        text_processor: F,
        root_dir: String,
    ) where
        F: Fn(&str) -> String + Sync + Send + 'static,
    {
        let processing_articles = self.processing_articles.clone();
        let active_threads = self.active_threads.clone();

        // Pause until we have some free threads
        loop {
            let active_threads = active_threads.load(Ordering::Relaxed);
            if active_threads < 10 * NUM_THREADS {
                break;
            }

            sleep(Duration::from_millis(50));
        }

        // Increase the number of active threads
        active_threads.fetch_add(1, Ordering::Relaxed);

        let root_dir = root_dir.clone();

        self.thread_pool.spawn(move || {
            // Add article to the list of articles being actively processed
            {
                let mut processing_articles = processing_articles.lock().unwrap();
                processing_articles.insert(article_id);

                if article_id % 10_000 == 0 {
                    println!("Processing the following files: {:?}", *processing_articles);
                }
            }

            // Process the text
            let text = (text_processor)(&text);

            // Write the text to a file
            write_file(&root_dir, &title, &text, article_id).unwrap();

            // Remove the article from the list of articles being processed
            {
                let mut processing_articles = processing_articles.lock().unwrap();
                processing_articles.remove(&article_id);
            }

            // Decrement number of active threads
            active_threads.fetch_sub(1, Ordering::Relaxed);
        });
    }
}
