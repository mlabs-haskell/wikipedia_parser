mod wikitext_parser;
mod xml_parser;

use std::error::Error;

use wikitext_parser::extract_text;
use xml_parser::XMLParser;

fn main() -> Result<(), Box<dyn Error>> {
    let mut xml_parser = XMLParser::new(extract_text, "data/wikipedia.xml")?;
    xml_parser.parse_xml()?;
    Ok(())
}
