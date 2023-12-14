use core::panic;
use std::collections::{HashMap, BTreeSet};
use std::fs::File;
use std::io::{Write, BufReader};
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
use regex::Regex;

const DIR: &str = "output/";
const NUM_THREADS: usize = 50;

pub struct XMLParser<F: Fn(&[u8]) -> String + Clone + Sync + Send + Copy + 'static> {
    text_processor: F,
    reader: Reader<BufReader<File>>,
    num_articles: usize,
    counts: Arc<Mutex<HashMap<String, usize>>>,
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
            counts: Arc::new(Mutex::new(HashMap::new())),
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
        }?;

        // Get and sort the counts
        self.thread_pool = None;
        let counts = self.counts.lock().unwrap();
        let mut sorted_counts: Vec<_> = counts.iter().collect();
        sorted_counts.sort_unstable_by_key(|(_, &c)| c);

        // Write the counts
        let mut file = File::create("counts.csv")?;
        for (s, c) in sorted_counts.iter().rev() {
            file.write_fmt(format_args!("{},{}\n", s, c))?;
        }

        Ok(())
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
        let title = str::from_utf8(&title)?;
        let title = title.to_string();
        if !title.starts_with("Wikipedia:") 
            && !title.starts_with("Portal:") 
            && !title.starts_with("File:") 
            && !title.starts_with("Template:") 
            && !title.starts_with("Category:") 
            && !title.starts_with("Draft:")
            && !title.starts_with("Module:")
            && !title.starts_with("MediaWiki:")
        {
            let counts = self.counts.clone();
            let processing_articles = self.processing_articles.clone();
            let active_threads = self.active_threads.clone();
            let text_processor = self.text_processor;

            // Pause until we have some free threads
            loop {
                {
                    let active_threads = active_threads.lock().unwrap();
                    if *active_threads < 2 * NUM_THREADS {
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
                let re = Regex::new(r"\{\{([^#<>\[\]\|\{\}]+)").unwrap();
                let text = (text_processor)(&text);

                // Count the number of templates
                {
                    let mut counts = counts.lock().unwrap();
                    for (_, [template_name]) in re.captures_iter(&text).map(|c| c.extract()) {
                        counts
                            .entry(template_name.trim().to_lowercase())
                            .and_modify(|c| *c += 1)
                            .or_insert(1);
                    }
                }

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
    let dir = String::from(DIR);
    let title = title.replace(|c: char| !c.is_alphanumeric(), "_");
    let title: String = title.chars().take(100).collect();
    let path = dir + &title + ".txt";

    let mut file = File::create(path);
    if let Ok(f) = file.as_mut() {
        f.write_all(text.as_bytes())?;
    }
    else {
        panic!("Could not write file: {}", title);
    }

    Ok(())
}