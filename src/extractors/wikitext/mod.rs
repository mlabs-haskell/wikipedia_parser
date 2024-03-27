mod template_transformers;
mod tree;
pub mod wikitext_parser;

pub fn extract(input: &[u8], title: &str) -> String {
    let text = wikitext_parser::extract_text(input);
    let tree = tree::Tree::from_string(title, &text);
    serde_json::ser::to_string_pretty(&tree).expect("failed to serialize")
}
