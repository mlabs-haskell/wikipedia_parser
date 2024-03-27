use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Page {
    pub title: String,
    pub links: Vec<Link>,
}

#[derive(Deserialize, Serialize)]
pub struct Link {
    pub target: String,
    pub label: String,
}

pub fn extract(input: &[u8], title: &str) -> String {
    let links = extract_links(input);
    let page = Page {
        title: title.to_owned(),
        links,
    };
    serde_json::ser::to_string_pretty(&page).expect("failed to serialize")
}

// We operate on u8 instead of chars to avoid the overhead of decoding UTF8.
// This works because UTF8 guarantees that the multibyte UTF8 sequences won't contain any ASCII
// characters. See the Backwards Compatibility section here:
// https://en.wikipedia.org/wiki/UTF-8#Comparison_with_other_encodings
fn extract_links(input: &[u8]) -> Vec<Link> {
    let mut links = Vec::new();

    let mut chunks = input.windows(2);
    'outer: loop {
        let chunk = match chunks.next() {
            Some(x) => x,
            None => break 'outer,
        };

        if chunk[0] == ('[' as u8) && chunk[1] == ('[' as u8) {
            let mut target_buffer = Vec::new();
            let mut label_buffer = Vec::new();

            let mut has_label = false;
            let mut current_buffer = &mut target_buffer;
            let mut nested_braces = 0;

            // skip the second [
            let _ = chunks.next();

            loop {
                let chunk = match chunks.next() {
                    Some(x) => x,
                    None => break 'outer,
                };

                if nested_braces == 0 && !has_label && chunk[0] == '|' as u8 {
                    current_buffer = &mut label_buffer;
                    has_label = true;
                    continue;
                }

                if chunk[0] == '{' as u8 {
                    nested_braces += 1;
                } else if chunk[0] == '}' as u8 {
                    nested_braces -= 1;
                }

                if chunk[0] == (']' as u8) && chunk[1] == (']' as u8) {
                    break;
                }

                current_buffer.push(chunk[0]);
            }

            // current_buffer points to label_buffer if there was a label
            // otherwise it points to target_buffer
            let label = String::from_utf8_lossy(&current_buffer).to_string();
            let target = String::from_utf8_lossy(&target_buffer).to_string();
            let link = Link { label, target };
            links.push(link)
        }
    }

    links
}
