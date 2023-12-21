use core::panic;
use std::collections::BTreeSet;
use std::fs::{File, create_dir_all};
use std::io::{Write, BufReader};
use std::path::Path;
use std::str;
use std::sync::{Mutex, Arc};
use std::thread::sleep;
use std::time::Duration;

use quick_xml::Error;
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use quick_xml::Result;

use rayon::{ThreadPool, ThreadPoolBuilder};

const DIR: &str = "output/";
const NUM_THREADS: usize = 32;

pub struct XMLParser<F: Fn(&[u8]) -> String + Clone + Sync + Send + Copy + 'static> {
    text_processor: F,
    reader: Reader<BufReader<File>>,
    num_articles: usize,
    thread_pool: Option<ThreadPool>,
    processing_articles: Arc<Mutex<BTreeSet<usize>>>,
    active_threads: Arc<Mutex<usize>>
}

impl<F: Fn(&[u8]) -> String + Clone + Sync + Send + Copy> XMLParser<F> {
    pub fn new(text_processor: F, filename: &str) -> Result<Self> {
        let reader = Reader::from_file(filename)?;
        let thread_pool = ThreadPoolBuilder::new().num_threads(NUM_THREADS).build().unwrap();
        Ok(Self {
            text_processor,
            reader,
            num_articles: 0,
            thread_pool: Some(thread_pool),
            processing_articles: Arc::new(Mutex::new(BTreeSet::new())),
            active_threads: Arc::new(Mutex::new(0))
        })
    }

    // Main XML parsing function
    pub fn parse_xml(&mut self) -> Result<()> {
        let mut buffer = Vec::new();
        match self.reader.read_event_into(&mut buffer) {
            Err(e) => self.terminate(e),
            Ok(Event::Start(e)) => 
                if e.name().into_inner() == b"mediawiki" {
                    self.parse_mediawiki()
                }
                else {
                    Err(Error::TextNotFound)
                },
            _ => Err(Error::TextNotFound)
        }
    }

    // Parse the body of the XML page
    fn parse_mediawiki(&mut self) -> Result<()> {
        let mut buffer = Vec::new();
        loop {
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
                },
                Ok(Event::Eof) => break,
                _ => ()
            }
        }

        Ok(())
    }

    fn parse_page(&mut self) -> Result<()> {
        let mut buffer = Vec::new();
        let mut title = Vec::new();
        let mut text = Vec::new();
        loop {
            match self.reader.read_event_into(&mut buffer) {
                Err(e) => self.terminate(e),
                Ok(Event::Empty(e)) => {
                    let tag = e.name().into_inner();
                    if tag == b"redirect" {
                        // We don't care about redirect pages
                        let mut garbage = Vec::new();
                        self.reader.read_to_end_into(QName(b"page"), &mut garbage)?;
                        return Ok(())
                    }
                },
                Ok(Event::Start(e)) => {
                    let tag = e.name().into_inner();
                    match tag {
                        b"title" => {
                            match self.reader.read_event_into(&mut title) {
                                Err(e) => self.terminate(e),
                                Ok(Event::Text(_)) => (),
                                _ => return Err(Error::TextNotFound)
                            }
                        },
                        b"ns" | b"id" => {
                            let mut garbage = Vec::new();
                            self.reader.read_to_end_into(QName(tag), &mut garbage)?;
                        },
                        b"revision" => self.parse_revision(&mut text)?,
                        _ => {
                            println!("Unknown tag: {}", String::from_utf8_lossy(tag));
                            let mut garbage = Vec::new();
                            self.reader.read_to_end_into(QName(tag), &mut garbage)?;
                        }
                    }
                },
                Ok(Event::End(e)) => if e.name().into_inner() == b"page" {
                    break
                },
                Ok(Event::Eof) => break,
                _ => ()
            }
        }
    
        // Skip technical articles about Wikipedia itself
        let title = String::from_utf8(title)?;
        if !title.starts_with("Wikipedia:") 
            && !title.starts_with("Portal:") 
            && !title.starts_with("File:") 
            && !title.starts_with("Template:") 
            && !title.starts_with("Category:") 
            && !title.starts_with("Draft:")
            && !title.starts_with("Module:")
            && !title.starts_with("MediaWiki:")
            && !title.to_lowercase().ends_with("(disambiguation)")
        {
            let processing_articles = self.processing_articles.clone();
            let active_threads = self.active_threads.clone();
            let text_processor = self.text_processor;

            // Pause until we have some free threads
            loop {
                {
                    let active_threads = active_threads.lock().unwrap();
                    if *active_threads < 32 * NUM_THREADS {
                        break;
                    }
                }
                sleep(Duration::from_millis(100));
            }

            let article_id = self.num_articles;
            self.num_articles += 1;

            // Increase the number of active threads
            {
                *active_threads.lock().unwrap() += 1;
            }

            self.thread_pool.as_mut().unwrap().spawn(move || {
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
                let title = format!("{}_{}", article_id, title);
                write_file(&title, &text).unwrap();

                // Remove the article from the list of articles being processed
                {
                    let mut processing_articles = processing_articles.lock().unwrap();
                    processing_articles.remove(&article_id);
                }

                // Decrement number of active threads
                {
                    *active_threads.lock().unwrap() -= 1;
                }
            });
        }
    
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
                        b"id" | 
                        b"parentid" | 
                        b"timestamp" | 
                        b"contributor" | 
                        b"minor" | 
                        b"comment" | 
                        b"model" | 
                        b"format" |
                        b"sha1" => {
                            let mut garbage = Vec::new();
                            self.reader.read_to_end_into(QName(tag), &mut garbage)?;
                        },
                        b"text" => {
                            let mut text_buffer = Vec::new();
                            match self.reader.read_event_into(&mut text_buffer) {
                                Err(e) => self.terminate(e),
                                Ok(Event::Text(e)) => {
                                    *text = e.to_vec();
                                },
                                _ => return Err(Error::TextNotFound)
                            }
                        },
                        _ => {
                            println!("Unknown tag: {}", String::from_utf8_lossy(tag));
                            let mut garbage = Vec::new();
                            self.reader.read_to_end_into(QName(tag), &mut garbage)?;
                        }
                    }
                },
                Ok(Event::End(e)) => if e.name().into_inner() == b"revision" {
                    break
                }
                Ok(Event::Eof) => return Err(Error::TextNotFound),
                _ => ()
            }
        }
    
        Ok(())
    }

    // Universal error
    fn terminate(&self, e: Error) -> ! {
        panic!("Error at position {}: {:?}", self.reader.buffer_position(), e)
    }
}

fn write_file(title: &str, text: &str) -> Result<()> {
    // Figure out where to write the file
    let dir = String::from(DIR);
    let title = title.replace(|c: char| !c.is_alphanumeric(), "_");
    let title: String = title.chars().take(100).collect();
    let sub_dir: String = title.chars().take(3).collect();
    let path = dir + "/" + &sub_dir + "/" + &title + ".txt";

    // Create the directories that will contain the file
    let path = Path::new(&path);
    let prefix = path.parent().unwrap();
    create_dir_all(prefix)?;

    // Write the file
    let mut file = File::create(path);
    if let Ok(f) = file.as_mut() {
        f.write_all(text.as_bytes())?;
    }
    else {
        panic!("Could not write file: {}", title);
    }

    Ok(())
}