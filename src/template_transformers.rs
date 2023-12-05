use std::collections::LinkedList;

const REMOVE_TEMPLATES: &[&str] = &[
    "further",
    "letter other reps",
    "certification cite ref",
    "clear",
    "charmap",
    "main article",
    "use",
    "good article",
    "infobox",
    "hidden",
    "efn",
    "see also",
    "music ratings",
    "awards table",
    "track listing",
    "sup",
    "div",
    "col",
    "album chart",
    "certification table",
    "notelist",
    "reflist",
    "cite",
    "short description",
    "about",
    "pp-protected",
    "technical reasons",
    "latin letter",
    "refn",
    "other uses",
    "pp",
    "toc",
    "main",
    "sfn",
    "ipa", // TODO: We can probably do something with IPA pronunciations
    "respell", // This is another IPA-related item
    "multiple image",
    "cleanup",
    "wikisource",
    "css",
    "additional citation needed",
    "inflation",
    "legend",
    "redirect",
    "undue weight section",
    "unreferenced section"
];

// TODO: Break this function into smaller pieces, or even move it into its own module
pub fn filter_templates(input: String) -> (bool, String) {
    // Handle templates that can always be totally removed
    let remove = REMOVE_TEMPLATES
        .iter()
        .any(|&s| input.to_lowercase().starts_with(s));
    if remove {
        return (false, String::new());
    }

    // Handle simple map cases
    let parts: Vec<_> = input.split('|').collect();
    let num_parts = parts.len();
    match parts[0].to_lowercase().as_str() {
        "sic" => return (true, parts[num_parts - 1].to_string()),
        "vr" => return (true, parts[num_parts - 1].to_string()),
        "script" => return (true, parts[num_parts - 1].to_string()),
        "midsize" => return (true, parts[num_parts - 1].to_string()),
        "'\"" => return (true, parts[num_parts - 1].to_string()),
        "convert" => return (false, parts[1].to_string() + " " + parts[2]),
        "sclass" => return (false, format!("{}-class {}", parts[1].trim(), parts[2].trim())),
        "uss" => {
            let s = if parts.len() == 2 {
                format!("USS {}", parts[1].trim())
            }
            else {
                format!("USS {} ({})", parts[1].trim(), parts[2].trim())
            };
            return (false, s);
        },
        _ => ()
    }
    if parts[0].to_lowercase().starts_with("angbr") {
        return (true, parts[num_parts - 1].to_string())
    }

    // Handle cases that need actual parsing
    if parts[0].to_lowercase().starts_with("quote") {
        let mut quote = "";
        let mut author = None;
        let mut source = None;
        for tag in &parts[1..] {
            let tag_pieces: Vec<_> = tag.split("=").map(|s| s.trim()).collect();
            match tag_pieces[0] {
                "quote" => quote = tag_pieces[1],
                "author" => author = Some(tag_pieces[1]),
                "source" => source = Some(tag_pieces[1]),
                _ => ()
            }
        }

        let caption = 
            if let Some((author, source)) = author.zip(source) {
                author.to_owned() + ", " + source
            }
            else if let Some(caption) = author.or(source) {
                caption.to_owned()
            }
            else {
                String::new()
            };
        
        let output = quote.to_owned() + &caption;
        return (true, output);
    }

    // Blockquotes are distinct from quote blocks.
    if parts[0].to_lowercase().starts_with("blockquote") {
        let mut text = "";
        let mut author = None;
        let mut title = None;
        let mut source = None;
        let mut character = None;
        for (i, tag) in parts[1..].iter().enumerate() {
            let tag_pieces: Vec<_> = tag.split("=").map(|s| s.trim()).collect();
            match tag_pieces[0] {
                "text" => text = tag_pieces[1],
                "author" => author = Some(tag_pieces[1]),
                "title" => title = Some(tag_pieces[1]),
                "source" => source = Some(tag_pieces[1]),
                "character" => character = Some(tag_pieces[1]),
                "sign" => author = Some(tag_pieces[1]),
                _ => match i {
                    0 => text = tag,
                    1 => author = Some(tag),
                    _ => ()
                }
            }
        }

        let mut caption_suffix_pieces = LinkedList::new();
        if let Some(s) = source {
            caption_suffix_pieces.push_front(s);
        }
        if let Some(t) = title {
            caption_suffix_pieces.push_front(t);
        }
        if let Some(a) = author {
            caption_suffix_pieces.push_front(a);
        }
        let caption_suffix = caption_suffix_pieces.into_iter().collect::<Vec<_>>().join(", ");

        let caption = if let Some(c) = character {
            if caption_suffix.is_empty() {
                c.to_owned()
            }
            else {
                format!("{c}, in {caption_suffix}")
            }
        }
        else {
            caption_suffix
        };
        
        let output = if caption.is_empty() {
            text.to_owned()
        }
        else {
            format!("\"{text}\"-{caption}")
        };
        return (true, output);
    }

    return (false, String::from("{{") + &input + "}}");
}