use std::collections::{LinkedList, HashMap};

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
    "unreferenced section",
    "relevance",
    "greek myth",
    "more citations needed",
    "agriculture",
    "by whom",
    "rp",
    "maplink",
    "alabama",
    "us census population",
    "expand",
    "citation needed"
];

const MAPPERS: &[(&str, &str)] = &[
    ("--)", ")"),
    ("'", "'"),
    ("'\"", "'\""),
    ("spaces", " "),
    ("snd", " - ")
];

const REPLACE_TEMPLATES: &[&str] = &[
    "sic",
    "vr",
    "script",
    "midsize",
    "nowrap",
    "transliteration",
    "section link",
    "crossreference",
    "lang"
];

const MONTHS: &[&str] = &[
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December"
];

const CONVERSION_SEPARATORS: &[&str] = &[
    "-",
    "\u{2013}",
    "and",
    "and(-)",
    "or",
    "to",
    "to(-)",
    "to about",
    "+/-",
    "\u{00B1}",
    "+",
    ",",
    ", and",
    ", or",
    "by",
    "x"
];

// Takes a template, processes it, and returns it and a bool flag 
// indicating if this output should be processed by the article parser again
pub fn filter_templates(input: String) -> (bool, String) {
    // Handle templates that can always be totally removed
    let remove = REMOVE_TEMPLATES
        .iter()
        .any(|&s| input.to_lowercase().starts_with(s));
    if remove {
        return (false, String::new());
    }

    // Handle templates that can be simply mapped
    let mapping = MAPPERS
        .iter()
        .find_map(|&(s, r)| {
            if s == &input {
                Some(r)
            }
            else {
                None
            }
        });
    if let Some(mapping) = mapping {
        return (false, mapping.to_string());
    }

    // Handle templates that should be replaced with its last portion
    let parts: Vec<_> = input.split('|').collect();
    let num_parts = parts.len();
    let replace = REPLACE_TEMPLATES
        .iter()
        .any(|&s| parts[0].to_lowercase().starts_with(s));
    if replace {
        return (true, parts[num_parts - 1].to_string());
    }
    if parts[0].to_lowercase().starts_with("angbr") {
        return (true, parts[num_parts - 1].to_string())
    }

    // Handle simple map cases
    match parts[0].to_lowercase().as_str() {
        "sclass" => return (false, format!("{}-class {}", parts[1].trim(), parts[2].trim())),
        "uss" | "hms" | "hmnzs" => {
            let s = if parts.len() == 2 {
                format!("{} {}", parts[0].trim(), parts[1].trim())
            }
            else {
                format!("{} {} ({})", parts[0].trim(), parts[1].trim(), parts[2].trim())
            };
            return (true, s);
        },
        "see below" => {
            let s = format!("(see {})", parts[1].trim());
            return (true, s);
        },
        "c." | "circa" => {
            let s = format!("{} {}", parts[0], parts[1]);
            return (true, s)
        },
        "ill" | "interlanguage link" => return (true, parts[1].trim().to_string()),
        "frac" | "fraction" => {
            match parts.len() {
                1 => return (false, "/".to_string()),
                2 => return (false, format!("1/{}", parts[1])),
                3 => return (false, format!("{}/{}", parts[1], parts[2])),
                4 => return (false, format!("{} {}/{}", parts[1], parts[2], parts[3])),
                _ => ()
            };
        },
        "cvt" | "convert" => {
            if CONVERSION_SEPARATORS.iter().any(|&s| s == parts[2]) {
                let s = format!("{} {} {} {}", parts[1], parts[2], parts[3], parts[4]);
                return (false, s);
            }
            else {
                let s = format!("{} {}", parts[1], parts[2]);
                return (false, s);
            }
        },
        _ => ()
    }

    // Handle cases that need actual parsing
    if parts[0].to_lowercase().starts_with("quote") {
        let tags = process_tags(&parts, &[]);

        let quote = tags["quote"];
        let author = tags.get("author");
        let source = tags.get("source");

        let caption = 
            if let Some((author, source)) = author.zip(source) {
                author.to_string() + ", " + source
            }
            else if let Some(caption) = author.or(source) {
                caption.to_string()
            }
            else {
                String::new()
            };
        
        let output = quote.to_owned() + &caption;
        return (true, output);
    }

    // Blockquotes are distinct from quote blocks.
    if parts[0].to_lowercase().starts_with("blockquote") {
        let tags = process_tags(&parts, &["text", "author"]);

        let text = tags["text"];
        let author = tags.get("author").or(tags.get("sign"));
        let title = tags.get("title");
        let source = tags.get("source");
        let character = tags.get("character");

        // Merge the source, title, and author pieces so long as they exist
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
        let caption_suffix = caption_suffix_pieces
            .into_iter()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>()
            .join(", ");

        // Prepend the character to the caption if it exists
        let caption = if let Some(c) = character {
            if caption_suffix.is_empty() {
                c.to_string()
            }
            else {
                format!("{c}, in {caption_suffix}")
            }
        }
        else {
            caption_suffix
        };
        
        // Format the quote by adding the source if it exists
        let output = if caption.is_empty() {
            text.to_owned()
        }
        else {
            format!("\"{text}\"-{caption}")
        };
        return (true, output);
    }

    // Parse "as of" blocks
    if parts[0].to_lowercase().starts_with("as of") {
        let tags = process_tags(&parts, &["year", "month", "day"]);

        let alt = tags.get("alt");
        let year = tags.get("year");
        let month = tags
            .get("month")
            .and_then(|s| s.parse::<usize>().ok().map(|i| MONTHS[i - 1]));
        let day = tags.get("day");
        let since = tags.get("since");
        let post = tags.get("post");

        if let Some(alt) = alt {
            return (true, alt.to_string());
        }
        else {
            let mut output = if since == Some(&"y") {
                "Since ".to_string()
            }
            else {
                parts[0].to_string() + " "
            };

            if let Some(day) = day {
                output += day;
                output += " ";
            }

            if let Some(month) = month {
                output += month;
                output += " ";
            }

            if let Some(year) = year {
                output += year;
                output += " ";
            }

            let mut output = output.trim().to_string();
            if let Some(post) = post {
                output += post;
                output += " ";
            }

            return (false, output);
        }
    }

    return (false, String::from("{{") + &input + "}}");
}

fn process_tags<'a, 'b>(
    parts: &'a [&'a str], 
    untagged_order: &'b [&'b str]
) -> HashMap<&'a str, &'a str> 
where
    'b: 'a
{
    let mut tags: HashMap<&str, &str> = HashMap::new();
    let mut untagged_count = 0;
    for tag in &parts[1..] {
        let tag_pieces: Vec<_> = tag.split("=").map(|s| s.trim()).collect();
        if tag_pieces.len() == 1 {
            let tag_name = untagged_order[untagged_count];
            tags.insert(tag_name, tag);
            untagged_count += 1;
        }
        else {
            let tag_name = tag_pieces[0];
            tags.insert(tag_name, tag_pieces[1]);
        }
    }

    tags
}