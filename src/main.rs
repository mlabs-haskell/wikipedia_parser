use clap::Parser;

use std::error::Error;

use wikipedia_parser::wikitext_parser::extract_text;
use wikipedia_parser::xml_parser::XMLParser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    output_dir: String
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut xml_parser = XMLParser::new(args.output_dir, extract_text, "data/wikipedia.xml")?;
    xml_parser.parse_xml()?;
    Ok(())
}
