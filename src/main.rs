use clap::{Parser, ValueEnum};

use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use wikipedia_parser::extractors;
use wikipedia_parser::par_file::ParFile;
use wikipedia_parser::xml_parser::XMLParser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    output_dir: String,
    #[arg(short, long)]
    extractor: Extractor,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Extractor {
    // Extract links graph
    Links,
    // Extract contents
    Contents,
}

const PAR_FILE_BLOCK_SIZE: usize = 100 * 1024 * 1024;
const PAR_FILE_QUEUE_SIZE: u64 = 1;
const PAR_FILE_NUM_THREADS: u64 = 16;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let extractor = match args.extractor {
        Extractor::Links => extractors::links::extract,
        Extractor::Contents => extractors::wikitext::extract,
    };

    let filename = "./data/enwiki-20231220-pages-articles-multistream.xml";

    let file = File::open(filename)?;
    let file_size = file.metadata()?.len();

    let file = ParFile::new(
        String::from(filename),
        PAR_FILE_BLOCK_SIZE as _,
        PAR_FILE_QUEUE_SIZE,
        PAR_FILE_NUM_THREADS,
    );

    let mut xml_parser = XMLParser::new(
        args.output_dir,
        extractor,
        BufReader::with_capacity(PAR_FILE_BLOCK_SIZE, file),
        file_size,
    )?;
    xml_parser.parse_xml()?;

    Ok(())
}
