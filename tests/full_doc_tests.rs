use wikipedia_parser::wikitext_parser::extract_text;

use std::fs;

const RAW_ARTICLE_DIR: &str = "resources/test/raw_articles/";
const PROCESSED_DIR: &str = "resources/test/processed_articles/";

// Load a file from the raw article directory
fn raw_file(filename: &str) -> String {
    let filename = RAW_ARTICLE_DIR.to_string() + filename + ".txt";
    fs::read_to_string(&filename)
        .expect("Should have been able to read the file")
}

// Load a file from the processed article directory
fn processed_file(filename: &str) -> String {
    let filename = PROCESSED_DIR.to_string() + filename + ".txt";
    fs::read_to_string(&filename)
        .expect("Should have been able to read the file")
}

// Load the raw and processed file of the given name, and make sure they are equal
fn test_full_doc(article_name: &str) {
    let raw = raw_file(article_name);
    let output = extract_text(raw.as_bytes());

    let processed = processed_file(article_name);

    assert_eq!(output.trim(), processed.trim());
}

#[test]
fn a() {
    test_full_doc("A");
}

#[test]
fn abraham_lincoln() {
    test_full_doc("Abraham Lincoln");
}

#[test]
fn baby_one_more_time() {
    test_full_doc("...Baby One More Time (album)");
}