use core::panic;
use std::fs::File;
use std::io::BufReader;
use std::str;
use std::time::Duration;
use std::time::SystemTime;

use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use quick_xml::Error;
use quick_xml::Result;

use crate::progress::progress;
use crate::work_queue::WorkQueue;

pub struct XMLParser {
    reader: Reader<BufReader<File>>,
    file_size: u64, // for tracking progress
    work_queue: WorkQueue,
}

impl XMLParser {
    pub fn new<F>(root_dir: String, text_processor: F, filename: &str) -> Result<Self>
    where
        F: Fn(&[u8], &str) -> String + Clone + Sync + Send + Copy + 'static,
    {
        let file = File::open(filename)?;
        let file_size = file.metadata()?.len();
        let reader = BufReader::with_capacity(4 * 1024 * 1024, file);
        let reader = Reader::from_reader(reader);
        let work_queue = WorkQueue::new(root_dir, text_processor);

        Ok(Self {
            reader,
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

        let mut progress = Progress {
            total: file_size,
            rate_divider: 1024.0 * 1024.0,
            rate_unit: "MB/s",
            window_length: Duration::from_secs(5),
            start: SystemTime::now(),
            start_count: 0,
        };
        loop {
            let pos = self.reader.buffer_position();

            let progress_str = progress.progress(pos as _, SystemTime::now());

            print!("Progress: {} \r", progress_str);

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

        self.work_queue.queue(text, title);

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
