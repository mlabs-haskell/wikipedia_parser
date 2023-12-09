use std::fs::File;
use std::io::{Write, BufReader};
use std::str;

use quick_xml::Error;
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use quick_xml::Result;

const DIR: &str = "output/";

pub struct XMLParser<F: Fn(&[u8]) -> String> {
    text_processor: F,
    reader: Reader<BufReader<File>>,
    num_articles: usize
}

impl<F: Fn(&[u8]) -> String> XMLParser<F> {
    pub fn new(text_processor: F, filename: &str) -> Result<Self> {
        let reader = Reader::from_file(filename)?;
        Ok(Self {
            text_processor,
            reader,
            num_articles: 0
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
        let title = str::from_utf8(&title)?;
        if !title.starts_with("Wikipedia:") 
            && !title.starts_with("Portal:") 
            && !title.starts_with("File:") 
            && !title.starts_with("Template:") 
            && !title.starts_with("Category:") 
            && !title.starts_with("Draft:")
        {
            if self.num_articles % 100 == 0 {
                println!("Processing file number {}: {}", self.num_articles, title);
            }
            let text = (self.text_processor)(&text);
            write_file(title, &text)?;
            self.num_articles += 1;
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