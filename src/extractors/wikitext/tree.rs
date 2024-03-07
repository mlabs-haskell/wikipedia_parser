use serde::{Deserialize, Serialize};
use std::{iter::Peekable, str::Lines};

#[derive(Deserialize, Serialize)]
pub struct Tree {
    section_name: String,
    text: String,
    children: Vec<Tree>,
}

impl Tree {
    pub fn from_string(title: &str, input: &str) -> Self {
        let lines = input.lines();
        let mut lines = lines.peekable();
        Self::from_string_worker(&mut lines, 1, title)
    }

    fn from_string_worker(lines: &mut Peekable<Lines>, level: usize, section_name: &str) -> Self {
        let mut text_acc = String::new();
        let mut children = Vec::new();

        while let Some(line) = lines.peek() {
            let line = line.trim();
            if line.is_empty() {
                lines.next();
                continue;
            }

            // Handle headers
            if line.starts_with("==") {
                // Count how many = signs there are for this header
                let new_header_depth = line.chars().take_while(|&c| c == '=').count();

                // Get the header name
                let header_chars: Vec<_> = line.chars().collect();
                let start_index = new_header_depth;
                let end_index = header_chars.len() - new_header_depth;
                let header_name: String = if start_index < end_index {
                    let header_name = &header_chars[start_index..end_index];
                    header_name.into_iter().collect()
                } else {
                    "Unknown".to_string()
                };

                // If there are more = signs than current level, parse child
                if new_header_depth > level {
                    lines.next();
                    let child =
                        Self::from_string_worker(lines, new_header_depth, header_name.trim());

                    // Don't add empty sections
                    if !child.text.is_empty() || !child.children.is_empty() {
                        children.push(child);
                    }
                }
                // Otherwise, there are no more children
                else {
                    break;
                }
            }
            // Accumulate non-header text
            else {
                text_acc += line;
                text_acc += "\n";
                lines.next();
            }
        }

        Self {
            section_name: section_name.to_string(),
            text: text_acc,
            children,
        }
    }
}
