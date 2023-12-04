use serde::{Deserialize, Serialize};
use std::str::Lines;

#[derive(Deserialize, Serialize)]
pub struct Tree {
    text: String,
    children: Vec<Tree>
}

impl Tree {
    pub fn new(text: String, children: Vec<Tree>) -> Self {
        Self {
            text,
            children
        }
    }

    pub fn from_string(input: String) -> Self {
        let mut lines = input.lines();
        Self::from_string_worker(&mut lines, 1)
    }

    fn from_string_worker(lines: &mut Lines, level: usize) -> Self {
        let mut text_acc = String::new();
        let mut children = Vec::new();

        while let Some(line) = lines.next() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("==") {
                let new_header_depth = line
                    .chars()
                    .take_while(|&c| c == '=')
                    .count();

                if new_header_depth > level {
                    let child = Self::from_string_worker(lines, new_header_depth);
                    children.push(child);
                }
                else {
                    break;
                }
            }

            else {
                text_acc += line;
            }
        }

        Self {
            text: text_acc,
            children
        }
    }
}