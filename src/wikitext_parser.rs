use html_escape::decode_html_entities;
use std::str::from_utf8;

pub fn extract_text(input: &[u8]) -> Vec<u8> {
    let input = from_utf8(input).unwrap();
    let decoded_html = decode_html_entities(input);
    Vec::from(decoded_html.as_ref().as_bytes())
}