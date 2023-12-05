use std::error::Error;

use wikipedia_parser::wikitext_parser::extract_text;
use wikipedia_parser::xml_parser::XMLParser;

fn main() -> Result<(), Box<dyn Error>> {
    let mut xml_parser = XMLParser::new(extract_text, "data/wikipedia.xml")?;
    xml_parser.parse_xml()?;
    Ok(())
}
