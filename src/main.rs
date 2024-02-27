use clap::{Parser, ValueEnum};

use std::error::Error;

use wikipedia_parser::extractors;
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

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let extractor = match args.extractor {
        Extractor::Links => extractors::links::extract,
        Extractor::Contents => extractors::wikitext::extract,
    };

    let mut xml_parser = XMLParser::new(
        args.output_dir,
        extractor,
        "./data/enwiki-20231220-pages-articles-multistream.xml",
    )?;
    xml_parser.parse_xml()?;

    Ok(())
}
