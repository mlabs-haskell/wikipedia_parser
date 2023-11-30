mod wikitext_parser;
mod xml_parser;

use wikitext_parser::extract_text;
use xml_parser::XMLParser;

fn main() {
    let mut xml_parser = XMLParser::new(extract_text, "data/wikipedia.xml").unwrap();
    xml_parser.parse_xml().unwrap();
}
