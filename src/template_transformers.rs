use std::collections::{LinkedList, HashMap};

const REMOVE_TEMPLATES: &[&str] = &[
    "#tag",
    "about",
    "according to whom",
    "additional citation needed",
    "agriculture",
    "alabama",
    "album chart",
    "algeria",
    "ambiguous",
    "anarchism",
    "anchor",
    "ancient greek religion",
    "anthropology",
    "apollo",
    "authority control",
    "awards table",
    "basic forms of government",
    "blp",
    "broader",
    "by whom",
    "certification cite ref",
    "certification table",
    "charmap",
    "chart",
    "citation",
    "cite",
    "clarify",
    "clarify",
    "cleanup",
    "clear",
    "cn",
    "col",
    "commons",
    "contains special characters",
    "css",
    "cyber",
    "date table sorting",
    "defaultsort",
    "div",
    "dts", // If this turns out to be used outside of a table, we'll need to handle it
    "dubious",
    "economic",
    "efn",
    "election box",
    "elucidate",
    "engvarb",
    "episode list",
    "esotericism",
    "etymology",
    "excerpt",
    "expand",
    "fact",
    "failed verification",
    "featured article",
    "flagcountry",
    "flagicon",
    "football box",
    "footballbox",
    "full citation needed",
    "further",
    "globalize",
    "good article",
    "goal",
    "greek myth",
    "hermeticism",
    "hidden",
    "image",
    "in lang",
    "inflation",
    "infobox",
    "ipa", // TODO: We can probably do something with IPA pronunciations
    "italic",
    "largest cities",
    "latin letter",
    "legend",
    "letter other reps",
    "listen",
    "location",
    "main article",
    "main",
    "maplink",
    "medical",
    "more citations needed",
    "multiple image",
    "multiple issues",
    "music ratings",
    "music",
    "n/a",
    "notelist",
    "other uses",
    "page needed",
    "party color",
    "party shading",
    "pb",
    "performance",
    "political",
    "pp-protected",
    "pp",
    "primary source",
    "redirect",
    "refimprove",
    "reflist",
    "refn",
    "relevance",
    "respell", // This is another IPA-related item
    "rp",
    "see also",
    "sfn",
    "short description",
    "shy",
    "small",
    "spaceflight",
    "specify",
    "sup",
    "table",
    "taxonbar",
    "technical reasons",
    "toc",
    "track listing",
    "undue weight section",
    "unreferenced section",
    "unreliable source",
    "update",
    "us census population",
    "use",
    "vague",
    "webarchive",
    "wikisource",
    "wiktionary",
    "yel"
];

const MAPPERS: &[(&str, &str)] = &[
    ("--)", ")"),
    ("'", "'"),
    ("' \"", "'\""),
    ("\" '", "\"'"),
    ("'\"", "'\""),
    ("spaces", " "),
    ("snd", " - "),
    ("nbsp", " "),
    ("'s", "'s"),
    ("en dash", "\u{2013}"),
    ("year", "2024"),
    ("!", "!")
];

const REPLACE_TEMPLATES: &[&str] = &[
    "avoid wrap",
    "center",
    "crossref",
    "crossreference",
    "flag",
    "isbn",
    "m+j",
    "keypress",
    "lang",
    "linktext",
    "midsize",
    "née",
    "notatypo",
    "nowrap",
    "oclc",
    "pslink",
    "script",
    "section link",
    "sic",
    "transliteration",
    "vr",
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
            if s == input.to_lowercase() {
                Some(r)
            }
            else {
                None
            }
        });
    if let Some(mapping) = mapping {
        return (false, mapping.to_string());
    }

    // Handle templates that should be replaced with its last parameter
    let parts: Vec<_> = input.split('|').collect();
    let num_parts = parts.len();
    let replace = REPLACE_TEMPLATES
        .iter()
        .any(|&s| parts[0].to_lowercase().starts_with(s));
    if replace {
        let parts: Vec<_> = parts.iter().filter(|&s| !s.contains('=')).collect();
        let num_parts = parts.len();
        return (true, parts[num_parts - 1].to_string());
    }
    if parts[0].to_lowercase().starts_with("angbr") {
        return (true, parts[num_parts - 1].to_string())
    }

    // Handle simple map cases
    match parts[0].to_lowercase().trim() {
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
            if parts.len() > 1 {
                let s = format!("{} {}", parts[0], parts[1]);
                return (true, s);
            }
            else {
                return (false, parts[0].to_string());
            }
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
        "r" => return (false, String::new()),
        "bce" | "ce" => {
            for part in &parts[1..] {
                if !part.contains('=') {
                    return (true, part.to_string() + " " + parts[0]);
                }
            }
        },
        "ietf rfc" => {
            let numbers: Vec<_> = parts
                .iter()
                .skip(1)
                .filter(|s| !s.contains('='))
                .map(|&s| s)
                .collect();
            return (false, format!("RFC {}", numbers.join(", ")));
        },
        "oldstyledateny" => return (true, parts[1].to_string()),
        "sortname" => return (true, parts[1].to_string() + " " + parts[2]),
        "mp" | "minor planet" => return (true, parts[1..].join(" ")),
        _ => ()
    }

    // Handle cases that need actual parsing
    if parts[0].to_lowercase().starts_with("quote") {
        let params = get_params(&parts, &["quote"]);

        let quote = params["quote"];
        let author = params.get("author");
        let source = params.get("source");

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
    if parts[0].to_lowercase().starts_with("blockquote") ||
        parts[0].to_lowercase().starts_with("quotation")
    {
        let params = get_params(&parts, &["text", "author"]);

        let text = params["text"];
        let author = params.get("author").or(params.get("sign"));
        let title = params.get("title");
        let source = params.get("source");
        let character = params.get("character");

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

    // Blockquotes are distinct from quote blocks.
    if parts[0].to_lowercase().starts_with("poemquote") 
        || parts[0].to_lowercase().starts_with("poem quote")
    {
        let params = get_params(&parts, &["text"]);

        let text = params["text"];
        let character = params.get("char");
        let author = params.get("author").or(params.get("sign"));
        let source = params.get("source");
        let title = params.get("title");

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
    if parts[0].to_lowercase().starts_with("as of") ||
        parts[0].to_lowercase().starts_with("asof")
    {
        let params = get_params(&parts, &["year", "month", "day"]);

        let alt = params.get("alt");
        let year = params.get("year");
        let month = params
            .get("month")
            .and_then(|s| s.parse::<usize>().ok().map(|i| MONTHS[i - 1]));
        let day = params.get("day");
        let since = params.get("since");
        let post = params.get("post");

        if let Some(alt) = alt {
            return (true, alt.to_string());
        }
        else {
            let mut output = if since == Some(&"y") {
                "Since ".to_string()
            }
            else {
                let as_of = if parts[0].chars().next() == Some('A') {
                    "As of"
                }
                else {
                    "as of"
                };
                as_of.to_string() + " "
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

    // Parse bibe verses blocks
    if parts[0].to_lowercase().starts_with("bibleverse") {
        let params = get_params(&parts, &["book", "verse", "version", "text"]);

        let book = params.get("book");
        let verse = params.get("verse");
        let text = params.get("text");

        if let Some(text) = text {
            return (true, text.to_string());
        }
        else if let Some(book) = book {
            if let Some(verse) = verse {
                return (true, format!("{}, {}", book, verse));
            }
            return (true, book.to_string());
        }
    }

    // Parse ordered lists
    if parts[0].to_lowercase().starts_with("ordered list") ||
        parts[0].to_lowercase().starts_with("unbulleted list")
    {
        let mut list_items = Vec::new();
        for &tag in &parts[1..] {
            let mut tag_pieces = tag.split('=');
            let first_piece = tag_pieces.next().unwrap();
            if first_piece.ends_with('\\') || first_piece.ends_with("{{") {
                list_items.push(tag);
            }
        }

        return (true, list_items.join("\n"));
    }

    // Parse coordinate templates
    if parts[0].to_lowercase().starts_with("coord") {
        let tag_pieces: Vec<_> = parts[1..]
            .iter()
            .filter(|&s| !s.contains('='))
            .collect();

        match tag_pieces.len() {
            2 => {
                let (lat_letter, lat) = if tag_pieces[0].starts_with('-') {
                    ('S', &tag_pieces[0][1..])
                }
                else {
                    ('N', *tag_pieces[0])
                };

                let (long_letter, long) = if tag_pieces[1].starts_with('-') {
                    ('W', &tag_pieces[0][1..])
                }
                else {
                    ('E', *tag_pieces[0])
                };

                return (
                    false, 
                    format!(
                        "{}\u{00B0}{} {}\u{00B0}{}", 
                        lat, 
                        lat_letter, 
                        long, 
                        long_letter
                    )
                );
            },
            4 => return (
                false, 
                format!(
                    "{}\u{00B0}{} {}\u{00B0}{}", 
                    tag_pieces[0], 
                    tag_pieces[1], 
                    tag_pieces[2], 
                    tag_pieces[3]
                )
            ),
            6 => return (
                false, 
                format!(
                    "{}\u{00B0}{}'{} {}\u{00B0}{}'{}", 
                    tag_pieces[0], 
                    tag_pieces[1], 
                    tag_pieces[2], 
                    tag_pieces[3],
                    tag_pieces[4],
                    tag_pieces[5]
                )
            ),
            8 => return (
                false, 
                format!(
                    "{}\u{00B0}{}'{}\"{} {}\u{00B0}{}'{}\"{}", 
                    tag_pieces[0], 
                    tag_pieces[1], 
                    tag_pieces[2], 
                    tag_pieces[3],
                    tag_pieces[4],
                    tag_pieces[5],
                    tag_pieces[6],
                    tag_pieces[7]
                )
            ),
            _ => ()
        }
    }

    // Get sorted item from sort templates
    if parts[0].to_lowercase() == "sort" {
        let params = get_params(&parts, &["1", "2"]);
        let sort_item = params.get("2").or(params.get("1"));
        return (true, sort_item.unwrap_or(&"").to_string());
    }

    // Get dates
    if parts[0].to_lowercase() == "start date" {
        // Get the tags we have and remove empty ones
        let params = get_params(&parts, &[
            "year", 
            "month", 
            "day", 
            "hour", 
            "minute", 
            "second", 
            "timezone"
        ]);
        let tags: HashMap<_, _> = params
            .into_iter()
            .filter(|(_, v)| !v.is_empty())
            .collect();

        // Collect the tags into variables
        let year = tags.get("year");
        let month = tags
            .get("month")
            .and_then(|s| s.parse::<usize>().ok().map(|i| MONTHS[i - 1]));
        let day = tags.get("day");
        let hour = tags.get("hour");
        let minute = tags.get("minute");
        let second = tags.get("second");
        let timezone = tags.get("timezone").map(|&tz| {
            if tz == "Z" {
                "UTC"
            }
            else {
                tz
            }
        });
        
        // Construct the time piecemeal
        let mut date_string = String::new();
        if let Some(year) = year {
            date_string += year;
        }
        
        if let Some(month) = month {
            if let Some(day) = day {
                date_string = format!("{} {}, {}", month, day, date_string);
            }
            else {
                date_string = format!("{} {}", month, date_string);
            }
        }

        // Hour can only be displayed if minute also exists
        if let Some((hour, minute)) = hour.and_then(|h| minute.map(|m| (h, m))) {
            let mut s = format!("{}:{}", hour, minute);
            if let Some(second) = second {
                s += ":";
                s += second;
            }
            date_string = format!("{}, {}", s, date_string);
        }

        if let Some (timezone) = timezone {
            date_string = format!("{} ({})", date_string, timezone);
        }

        return (false, date_string)
    }

    // Parse lang templates
    if parts[0].to_lowercase().starts_with("lang") {
        // Filter named parameters
        let tag_pieces: Vec<_> = parts[1..]
            .iter()
            .filter(|&s| !s.contains('='))
            .collect();
        return (true, tag_pieces[tag_pieces.len() - 1].to_string());
    }

    // Parse athlete flag templates
    if parts[0].to_lowercase().starts_with("flagathlete") {
        let params = get_params(&parts, &["name", "country"]);
        let name = params["name"];
        let country = params["country"];
        return (true, format!("{} ({})", name, country));
    }

    // Parse birthdate and year templates
    if parts[0].to_lowercase() == "birth date and age" ||
        parts[0].to_lowercase() == "bda"
    {
        let params = get_params(&parts, &["year", "month", "day"]);
        let year = params["year"];
        let month = params["month"];
        let day = params["day"];

        let month = month
            .parse::<usize>()
            .map(|m| MONTHS[m - 1])
            .unwrap_or(month);

        return (false, format!("{month} {day}, {year}"));
    }

    // Parse rollover abbreviations
    if parts[0].to_lowercase() == "abbr" {
        let params = get_params(&parts, &["text", "meaning"]);
        let text = params["text"];
        let meaning = params["meaning"];
        return (true, format!("{} ({})", text, meaning));
    }

    // Parse Japanese translation helpers
    if parts[0].to_lowercase() == "nihongo" {
        let params = get_params(&parts, &["english", "kanji", "romaji", "extra1", "extra2"]);
        let params: HashMap<_, _> = params
            .into_iter()
            .filter(|(_, s)| !s.is_empty())
            .collect();

        let english = params.get("english");
        let kanji = params["kanji"];
        let romaji = params.get("romaji");
        let extra1 = params.get("extra1");
        let extra2 = params.get("extra2");

        // Determine main display of text
        let formatted_text = if let Some(english) = english {
            english
        }
        else if let Some(romaji) = romaji {
            romaji
        }
        else {
            ""
        };
        let mut formatted_text = formatted_text.to_string() + "(" + kanji;

        // Add romaji in parens if english wasn't present
        if english.is_none() {
            if let Some(romaji) = romaji {
                formatted_text += ", ";
                formatted_text += romaji;
            }
        }

        // Add extra1 in parens if present
        if let Some(extra1) = extra1 {
            formatted_text += ", ";
            formatted_text += extra1;
        }

        // Terminate parens and add extra2 if present
        formatted_text += ")";
        if let Some(extra2) = extra2 {
            formatted_text += " ";
            formatted_text += extra2;
        }

        return (true, formatted_text);
    }

    (false, String::from("{{") + &input + "}}")
}

// Get the template parameters. 
// If a parameter is unnamed, give it the next unused name from unnamed_order
fn get_params<'a, 'b>(
    parts: &'a [&'a str], 
    unnamed_order: &'b [&'b str]
) -> HashMap<&'a str, &'a str> 
where
    'b: 'a
{
    let mut params: HashMap<&str, &str> = HashMap::new();
    let mut unnamed_count = 0;
    for param in &parts[1..] {
        let param_pieces: Vec<_> = param.split('=').map(|s| s.trim()).collect();
        if param_pieces.len() == 1 {
            if unnamed_count < unnamed_order.len() {
                let tag_name = unnamed_order[unnamed_count];
                params.insert(tag_name, param);
                unnamed_count += 1;
            }
        }
        else {
            let tag_name = param_pieces[0];
            params.insert(tag_name, param_pieces[1]);
        }
    }

    params
}