use wikipedia_parser::extractors::wikitext::extract;

use std::fs;

const RAW_ARTICLE_DIR: &str = "resources/test/raw_articles/";
const PROCESSED_DIR: &str = "resources/test/processed_articles/";

// Load a file from the raw article directory
fn raw_file(filename: &str) -> Vec<u8> {
    let filename = RAW_ARTICLE_DIR.to_string() + filename + ".txt";
    fs::read(&filename).expect("Should have been able to read the file")
}

// Load a file from the processed article directory
fn processed_file(filename: &str) -> String {
    let filename = PROCESSED_DIR.to_string() + filename + ".txt";
    fs::read_to_string(&filename).expect("Should have been able to read the file")
}

// Load the raw and processed file of the given name, and make sure they are equal
fn test_full_doc(article_name: &str) {
    let raw = raw_file(article_name);
    let output = extract(&raw, &article_name);

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
fn achilles() {
    test_full_doc("Achilles");
}

#[test]
fn alabama() {
    test_full_doc("Alabama");
}

#[test]
fn an_american_in_paris() {
    test_full_doc("An American in Paris");
}

#[test]
fn anthropology() {
    test_full_doc("Anthropology");
}

#[test]
fn apollo_8() {
    test_full_doc("Apollo 8");
}

#[test]
fn baby_one_more_time() {
    test_full_doc("...Baby One More Time (album)");
}
