use lazy_static::lazy_static;

use keshvar::IOC;
use std::collections::{LinkedList, HashMap, HashSet};

const REMOVE_TEMPLATES: &[&str] = &[
    // "#tag",
    // "0",
    // "about",
    // "according to whom",
    // "additional citation needed",
    // "agriculture",
    // "alabama",
    // "album chart",
    // "algeria",
    // "ambiguous",
    // "american football roster",
    // "anarchism",
    // "anchor",
    // "ancient greek religion",
    // "anthropology",
    // "apollo",
    // "authority control",
    // "awards table",
    // "basic forms of government",
    // "blp",
    // "broader",
    // "by whom",
    // "canadian party colour",
    // "cbb roster",
    // "cbb schedule entry",
    // "cbignore",
    // "certification cite ref",
    // "certification table",
    // "cfb schedule entry",
    // "chset-cell1",
    // "charmap",
    // "chart",
    // "citation",
    // "cite",
    // "clade",
    // "clarify",
    // "cleanup",
    // "clear",
    // "cn",
    // "col-",
    // "commons",
    // "contains special characters",
    // "coord missing",
    // "css",
    // "ct",
    // "cyber",
    // "date table sorting",
    // "dead link",
    // "decrease",
    // "defaultsort",
    // "detailslink",
    // "disambiguation",
    // "distinguish",
    // "div",
    // "dts", // If this turns out to be used outside of a table, we'll need to handle it
    // "dubious",
    // "economic",
    // "efn",
    // "efs player",
    // "election box",
    // "elucidate",
    // "engvarb",
    // "episode list", // This is a table, but we can probably extract information from it
    // "esotericism",
    // "etymology",
    // "excerpt",
    // "expand",
    // "fact",
    // "failed verification",
    // "fbaicon",
    // "featured article",
    // "fhgoal",
    // "flagcountry",
    // "flagdeco",
    "flagicon",
    // "football box",
    // "footballbox",
    // "formatnum",
    // "france metadata wikidata",
    // "full citation needed",
    // "further",
    // "globalize",
    // "good article",
    // "goal",
    // "greek myth",
    // "harvid",
    // "harvnb",
    // "hermeticism",
    // "hidden",
    // "hs",
    // "image",
    // "imdb",
    // "in lang",
    // "inflation",
    // "infobox",
    // "ipa", // TODO: We can probably do something with IPA pronunciations
    // "italic",
    // "largest cities",
    // "latin letter",
    // "leagueicon",
    // "legend",
    // "letter other reps",
    // "listen",
    // "location map",
    // "lomp",
    // "london gazette",
    // "main",
    // "maplink",
    // "marriage", // Only meant to appear in infoboxes
    // "math",
    // "medal", 
    // "medical",
    // "more citations needed",
    // "multiple image",
    // "multiple issues",
    // "music ratings",
    // "music",
    // "nat fs g player",
    // "nhle",
    // "nhrp row",
    // "notelist",
    // "nts",
    // "official website",
    // "oneleg",
    // "open access",
    // "other uses",
    // "page needed",
    // "party color",
    // "party shading",
    // "party stripe",
    // "pb",
    // "pengoal",
    // "performance",
    // "plainlist",
    // "political",
    // "portal",
    // "pp-protected",
    // "pp",
    // "presfoot",
    // "preshead",
    // "presrow",
    // "primary source",
    // "rating",
    // "redirect",
    // "refimprove",
    "reflist",
    // "refn",
    // "relevance",
    // "respell", // This is another IPA-related item
    // "rp",
    // "s-aft",
    // "s-bef",
    // "s-end",
    // "s-start",
    // "s-ttl",
    // "see also",
    "sfn",
    // "shipwreck list item",
    "short description",
    // "shy",
    // "single chart",
    // "singlechart",
    // "small",
    // "spaceflight",
    // "speciesbox",
    // "specify",
    // "subon",
    // "succession box",
    // "sup",
    // "table",
    // "taxobox",
    // "taxonbar",
    // "technical reasons",
    // "toc",
    // "track listing",
    // "undue weight section",
    // "unreferenced",
    // "unreferenced section",
    // "unreliable source",
    // "update",
    // "url",
    // "us census population",
    // "usa",
    // "use",
    // "vague",
    // "webarchive",
    // "wikisource",
    // "wiktionary",
    // "yel"
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

lazy_static! {
    static ref MAPPERS: HashMap<String, String> = {
        [
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
            ("!", "!"),
            ("no", "No"),
            ("yes", "Yes"),
            ("yes2", "Yes"),
            ("yes-no", "Yes"),
            ("won", "Won"),
            ("nom", "Nominated"),
            ("n/a", "N/A"),
            ("r", ""),
            ("·", "·"),
            ("gold1", "1"),
            ("silver2", "2"),
            ("col", ""),
            ("for", ""),
            ("end", ""),
            ("ya", "")
        ].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    };

    static ref REPLACE_LAST: HashSet<String> = {
        [
            "avoid wrap",
            "angbr",
            "australian party style",
            "center",
            "crossref",
            "crossreference",
            "fb",
            "fbw",
            "flag",
            "flagg",
            "flagu",
            "flatlist",
            "isbn",
            "m+j",
            "keypress",
            "lang",
            "linktext",
            "medalsport",
            "midsize",
            "née",
            "notatypo",
            "nowrap",
            "oclc",
            "pslink",
            "script",
            "section link",
            "sic",
            "transl",
            "vr",
        ].iter().map(|s| s.to_string()).collect()
    };

    static ref REPLACE_FIRST: HashSet<String> = {
        [
            "cr",
            "esc",
            "ill",
            "interlanguage link",
            "oldstyledateny",
            "stn",
            "station"
        ].iter().map(|s| s.to_string()).collect()
    };

    static ref MERGE_WITH_SPACES: HashSet<String> = {
        [
            "hlist",
            "linktext",
            "minor planet",
            "mp",
            "nflplayer",
            "post-nominals",
            "ship",
        ].iter().map(|s| s.to_string()).collect()
    };

    static ref CONVERSION_SEPARATORS: HashSet<String> = {
        [
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
        ].iter().map(|s| s.to_string()).collect()
    };
}

// Takes a template, processes it, and returns it and a bool flag 
// indicating if this output should be processed by the article parser again
pub fn filter_templates(input: &str) -> Option<String> {
    // Handle templates that can be simply mapped to a constant
    // let mapping = MAPPERS.get(&input.to_lowercase());
    // if let Some(mapping) = mapping {
    //     return (false, mapping.to_string());
    // }

    // Get the template name and its params
    let parts: Vec<_> = input.split('|').map(|s| s.trim()).collect();
    let template_name: Vec<_> = parts[0]
        .split(' ')
        .filter(|s| !s.is_empty())
        .collect();
    let template_name = template_name.join(" ").to_lowercase();
    let params = get_params(&parts[1..]);

    // Handle any template that should be replaced with its last parameter
    // let replace = REPLACE_LAST.contains(&template_name);
    // if replace {
    //     let num_params = unnamed_params.len();
    //     if num_params > 0 {
    //         return (true, unnamed_params[num_params - 1].to_string());
    //     }
    //     else {
    //         return (false, String::new());
    //     }
    // }

    // Handle any template that should be replaced with its first parameter
    // let replace = REPLACE_FIRST.contains(&template_name);
    // if replace {
    //     let num_params = unnamed_params.len();
    //     if num_params > 0 {
    //         return (true, unnamed_params.get(0)?.to_string());
    //     }
    //     else {
    //         return (false, String::new());
    //     }
    // }

    // Handle any template where the unnamed params can be joined with spaces
    // let replace = MERGE_WITH_SPACES.contains(&template_name);
    // if replace {
    //     return (true, unnamed_params.join(" "));
    // }

    // Handle simple parsing cases
    match template_name.trim() {
        // "sclass" => return (false, format!("{}-class {}", unnamed_params.get(0)?, unnamed_params.get(1)?)),
        // "uss" | "hms" | "hmnzs" => {
        //     let s = if parts.len() == 2 {
        //         format!("{} {}", parts[0], parts[1])
        //     }
        //     else {
        //         format!("{} {} ({})", parts.get(0)?, parts.get(1)?, parts.get(2)?)
        //     };
        //     return (true, s);
        // },
        // "see below" => return (true, format!("(see {})", unnamed_params.get(0)?)),
        // "c." | "circa" => {
        //     if parts.len() > 1 {
        //         let s = format!("{} {}", parts[0], parts[1]);
        //         return (true, s);
        //     }
        //     else {
        //         return (false, parts[0].to_string());
        //     }
        // },
        // "frac" | "fraction" => {
        //     match params.len() {
        //         0 => return (false, "/".to_string()),
        //         1 => return (false, format!("1/{}", unnamed_params[0])),
        //         2 => return (false, format!("{}/{}", unnamed_params[0], unnamed_params[1])),
        //         3 => return (false, format!(
        //             "{} {}/{}", 
        //             unnamed_params[0], 
        //             unnamed_params[1], 
        //             unnamed_params[2])
        //         ),
        //         _ => ()
        //     };
        // },
        "cvt" | "convert" => {
            let separator = params.get("2")?;
            if CONVERSION_SEPARATORS.contains(*separator) {
                let s = format!(
                    "{} {} {} {}", 
                    params.get("1")?, 
                    params.get("2")?, 
                    params.get("3")?, 
                    params.get("4")?
                );
                return Some(s);
            }
            else {
                let s = format!("{} {}", params.get("1")?, params.get("2")?);
                return Some(s);
            }
        },
        // "bce" | "ce" => return (true, unnamed_params.get(0)?.to_string() + " " + parts.get(0)?),
        // "ietf rfc" => return (false, format!("RFC {}", unnamed_params.join(", "))),
        // "sortname" => return (true, unnamed_params.get(0)?.to_string() + " " + unnamed_params.get(1)?),
        // "jct" => {         
        //     let pairs = unnamed_params.chunks(2);
        //     let highways: Vec<_> = pairs
        //         .into_iter()
        //         .map(|pair| pair.join("-"))
        //         .collect();

        //     return (true, highways.join("/"))
        // },
        // "mlbplayer" => return (true, unnamed_params.get(1)?.to_string()),
        // "fbu" | "fb-rt" => return (true, unnamed_params.get(1)?.to_string()),
        // "flagmedalist" => return (true, format!("{} ({})", unnamed_params.get(0)?, unnamed_params.get(1)?)),
        // "party name with colour" | "party name with color" => 
        //     return (true, unnamed_params.get(1)?.to_string()),
        // "suboff" => return (true, unnamed_params.get(0).unwrap_or(&"").to_string()),
        // "val" => {
        //     match unnamed_params.len() {
        //         1 | 3 => return (true, unnamed_params.get(0)?.to_string()),
        //         2 => return (
        //             true, 
        //             unnamed_params.get(0)?.to_string() + "±" + unnamed_params.get(1)?
        //         ),
        //         _ => ()
        //     }
        // },
        // "composition bar" => return (
        //     true, 
        //     format!("{}/{}", unnamed_params.get(0)?, unnamed_params.get(1)?)
        // ),
        // "nfl year" => {
        //     if unnamed_params.len() == 1 {
        //         return (true, unnamed_params.get(0)?.to_string());
        //     }
        //     if unnamed_params.len() == 2 {
        //         return (true, format!("{}-{}", unnamed_params.get(0)?, unnamed_params.get(1)?));
        //     }
        // },
        _ => ()
    }

    // Handle cases that need actual parsing
    // if template_name == "quote" {
    //     let params = get_params(&params, &["quote"]);

    //     let quote = params["quote"];
    //     let author = params.get("author");
    //     let source = params.get("source");

    //     let caption = 
    //         if let Some((author, source)) = author.zip(source) {
    //             author.to_string() + ", " + source
    //         }
    //         else if let Some(caption) = author.or(source) {
    //             caption.to_string()
    //         }
    //         else {
    //             String::new()
    //         };
        
    //     let output = quote.to_owned() + &caption;
    //     return (true, output);
    // }

    // // Blockquotes are distinct from quote blocks.
    // if template_name == "blockquote" ||
    //     template_name == "quotation"
    // {
    //     let params = get_params(&params, &["text", "author"]);

    //     let text = params["text"];
    //     let author = params.get("author").or(params.get("sign"));
    //     let title = params.get("title");
    //     let source = params.get("source");
    //     let character = params.get("character");

    //     // Merge the source, title, and author pieces so long as they exist
    //     let mut caption_suffix_pieces = LinkedList::new();
    //     if let Some(s) = source {
    //         caption_suffix_pieces.push_front(s);
    //     }
    //     if let Some(t) = title {
    //         caption_suffix_pieces.push_front(t);
    //     }
    //     if let Some(a) = author {
    //         caption_suffix_pieces.push_front(a);
    //     }
    //     let caption_suffix = caption_suffix_pieces
    //         .into_iter()
    //         .map(|s| s.to_owned())
    //         .collect::<Vec<_>>()
    //         .join(", ");

    //     // Prepend the character to the caption if it exists
    //     let caption = if let Some(c) = character {
    //         if caption_suffix.is_empty() {
    //             c.to_string()
    //         }
    //         else {
    //             format!("{c}, in {caption_suffix}")
    //         }
    //     }
    //     else {
    //         caption_suffix
    //     };
        
    //     // Format the quote by adding the source if it exists
    //     let output = if caption.is_empty() {
    //         text.to_owned()
    //     }
    //     else {
    //         format!("\"{text}\"-{caption}")
    //     };
    //     return (true, output);
    // }

    // // Handle poems
    // if template_name == "poemquote"
    //     || template_name == "poem quote"
    // {
    //     let params = get_params(&params, &["text"]);

    //     let text = params["text"];
    //     let character = params.get("char");
    //     let author = params.get("author").or(params.get("sign"));
    //     let source = params.get("source");
    //     let title = params.get("title");

    //     // Merge the source, title, and author pieces so long as they exist
    //     let mut caption_suffix_pieces = LinkedList::new();
    //     if let Some(s) = source {
    //         caption_suffix_pieces.push_front(s);
    //     }
    //     if let Some(t) = title {
    //         caption_suffix_pieces.push_front(t);
    //     }
    //     if let Some(a) = author {
    //         caption_suffix_pieces.push_front(a);
    //     }
    //     let caption_suffix = caption_suffix_pieces
    //         .into_iter()
    //         .map(|s| s.to_owned())
    //         .collect::<Vec<_>>()
    //         .join(", ");

    //     // Prepend the character to the caption if it exists
    //     let caption = if let Some(c) = character {
    //         if caption_suffix.is_empty() {
    //             c.to_string()
    //         }
    //         else {
    //             format!("{c}, in {caption_suffix}")
    //         }
    //     }
    //     else {
    //         caption_suffix
    //     };
        
    //     // Format the quote by adding the source if it exists
    //     let output = if caption.is_empty() {
    //         text.to_owned()
    //     }
    //     else {
    //         format!("\"{text}\"-{caption}")
    //     };
    //     return (true, output);
    // }

    // // Parse "as of" blocks
    // if template_name == "as of" ||
    //     template_name == "asof"
    // {
    //     let params = get_params(&params, &["year", "month", "day"]);

    //     let alt = params.get("alt");
    //     let year = params.get("year");
    //     let month = params
    //         .get("month")
    //         .and_then(|s| s.parse::<usize>().ok().map(|i| MONTHS[i - 1]));
    //     let day = params.get("day");
    //     let since = params.get("since");
    //     let post = params.get("post");

    //     if let Some(alt) = alt {
    //         return (true, alt.to_string());
    //     }
    //     else {
    //         let mut output = if since == Some(&"y") {
    //             "Since ".to_string()
    //         }
    //         else {
    //             let as_of = if parts.get(0)?.chars().next() == Some('A') {
    //                 "As of"
    //             }
    //             else {
    //                 "as of"
    //             };
    //             as_of.to_string() + " "
    //         };

    //         if let Some(day) = day {
    //             output += day;
    //             output += " ";
    //         }

    //         if let Some(month) = month {
    //             output += month;
    //             output += " ";
    //         }

    //         if let Some(year) = year {
    //             output += year;
    //             output += " ";
    //         }

    //         let mut output = output.trim().to_string();
    //         if let Some(post) = post {
    //             output += post;
    //             output += " ";
    //         }

    //         return (false, output);
    //     }
    // }

    // // Parse bibe verses blocks
    // if template_name == "bibleverse" {
    //     let params = get_params(&params, &["book", "verse", "version", "text"]);

    //     let book = params.get("book");
    //     let verse = params.get("verse");
    //     let text = params.get("text");

    //     if let Some(text) = text {
    //         return (true, text.to_string());
    //     }
    //     else if let Some(book) = book {
    //         if let Some(verse) = verse {
    //             return (true, format!("{}, {}", book, verse));
    //         }
    //         return (true, book.to_string());
    //     }
    // }

    // // Parse ordered lists
    // if template_name == "ordered list" ||
    //     template_name == "unbulleted list" ||
    //     template_name == "ubl"
    // {
    //     let mut list_items = Vec::new();
    //     for &param in params {
    //         let mut param_pieces = param.split('=');
    //         let first_piece = param_pieces.next().unwrap();
    //         if first_piece.ends_with('\\') || first_piece.ends_with("{{") {
    //             list_items.push(param);
    //         }
    //     }

    //     return (true, list_items.join("\n"));
    // }

    // // Parse coordinate templates
    // if template_name == "coord" ||
    //     template_name == "location"
    // {
    //     let tag_pieces: Vec<_> = params
    //         .iter()
    //         .filter(|&s| !s.contains('='))
    //         .collect();

    //     match tag_pieces.len() {
    //         2 => {
    //             let (lat_letter, lat) = if tag_pieces.get(0)?.starts_with('-') {
    //                 ('S', &tag_pieces.get(0)?[1..])
    //             }
    //             else {
    //                 ('N', *tag_pieces.get(0)?)
    //             };

    //             let (long_letter, long) = if tag_pieces.get(1)?.starts_with('-') {
    //                 ('W', &tag_pieces.get(0)?[1..])
    //             }
    //             else {
    //                 ('E', *tag_pieces.get(0)?)
    //             };

    //             return (
    //                 false, 
    //                 format!(
    //                     "{}\u{00B0}{} {}\u{00B0}{}", 
    //                     lat, 
    //                     lat_letter, 
    //                     long, 
    //                     long_letter
    //                 )
    //             );
    //         },
    //         4 => return (
    //             false, 
    //             format!(
    //                 "{}\u{00B0}{} {}\u{00B0}{}", 
    //                 tag_pieces.get(0)?, 
    //                 tag_pieces.get(1)?, 
    //                 tag_pieces.get(2)?, 
    //                 tag_pieces.get(3)?
    //             )
    //         ),
    //         6 => return (
    //             false, 
    //             format!(
    //                 "{}\u{00B0}{}'{} {}\u{00B0}{}'{}", 
    //                 tag_pieces.get(0)?, 
    //                 tag_pieces.get(1)?, 
    //                 tag_pieces.get(2)?, 
    //                 tag_pieces.get(3)?,
    //                 tag_pieces.get(4)?,
    //                 tag_pieces.get(5)?
    //             )
    //         ),
    //         8 => return (
    //             false, 
    //             format!(
    //                 "{}\u{00B0}{}'{}\"{} {}\u{00B0}{}'{}\"{}", 
    //                 tag_pieces.get(0)?, 
    //                 tag_pieces.get(1)?, 
    //                 tag_pieces.get(2)?, 
    //                 tag_pieces.get(3)?,
    //                 tag_pieces.get(4)?,
    //                 tag_pieces.get(5)?,
    //                 tag_pieces.get(6)?,
    //                 tag_pieces.get(7)?
    //             )
    //         ),
    //         _ => ()
    //     }
    // }

    // // Get sorted item from sort templates
    // if template_name == "sort" {
    //     let params = get_params(&params, &["1", "2"]);
    //     let sort_item = params.get("2").or(params.get("1"));
    //     return (true, sort_item.unwrap_or(&"").to_string());
    // }

    // // Get dates
    // if template_name == "start date" ||
    //     template_name == "start date and age" ||
    //     template_name == "end date"
    // {
    //     // Get the tags we have and remove empty ones
    //     let params = get_params(&params, &[
    //         "year", 
    //         "month", 
    //         "day", 
    //         "hour", 
    //         "minute", 
    //         "second", 
    //         "timezone"
    //     ]);
    //     let tags: HashMap<_, _> = params
    //         .into_iter()
    //         .filter(|(_, v)| !v.is_empty())
    //         .collect();

    //     // Collect the tags into variables
    //     let year = tags.get("year");
    //     let month = tags
    //         .get("month")
    //         .and_then(|s| s.parse::<usize>().ok().map(|i| MONTHS[i - 1]));
    //     let day = tags.get("day");
    //     let hour = tags.get("hour");
    //     let minute = tags.get("minute");
    //     let second = tags.get("second");
    //     let timezone = tags.get("timezone").map(|&tz| {
    //         if tz == "Z" {
    //             "UTC"
    //         }
    //         else {
    //             tz
    //         }
    //     });
        
    //     // Construct the time piecemeal
    //     let mut date_string = String::new();
    //     if let Some(year) = year {
    //         date_string += year;
    //     }
        
    //     if let Some(month) = month {
    //         if let Some(day) = day {
    //             date_string = format!("{} {}, {}", month, day, date_string);
    //         }
    //         else {
    //             date_string = format!("{} {}", month, date_string);
    //         }
    //     }

    //     // Hour can only be displayed if minute also exists
    //     if let Some((hour, minute)) = hour.and_then(|h| minute.map(|m| (h, m))) {
    //         let mut s = format!("{}:{}", hour, minute);
    //         if let Some(second) = second {
    //             s += ":";
    //             s += second;
    //         }
    //         date_string = format!("{}, {}", s, date_string);
    //     }

    //     if let Some (timezone) = timezone {
    //         date_string = format!("{} ({})", date_string, timezone);
    //     }

    //     return (false, date_string)
    // }

    // // Get film dates
    // if template_name == "film date" {
    //     // Get the tags we have and remove empty ones
    //     let params = get_params(&params, &["year", "month", "day"]);
    //     let tags: HashMap<_, _> = params
    //         .into_iter()
    //         .filter(|(_, v)| !v.is_empty())
    //         .collect();

    //     // Collect the tags into variables
    //     let year = tags["year"];
    //     let month = tags
    //         .get("month")
    //         .and_then(|s| s.parse::<usize>().ok().map(|i| MONTHS[i - 1]));
    //     let day = tags.get("day");
        
    //     // Construct the date piecemeal
    //     let mut date_string = year.to_string();
        
    //     if let Some(day) = day {
    //         date_string = format!("{}, {}", day, date_string);
    //     }

    //     if let Some(month) = month {
    //         date_string = format!("{} {}", month, date_string);
    //     }

    //     return (false, date_string);
    // }

    // // Parse athlete flag templates
    // if template_name == "flagathlete" {
    //     let params = get_params(&params, &["name", "country"]);
    //     let name = params["name"];
    //     let country = params["country"];
    //     return (true, format!("{} ({})", name, country));
    // }

    // // Parse AllMusic links templates
    // if template_name == "allmusic" {
    //     let params = get_params(&params, &["1", "2", "title"]);
    //     let text = params
    //         .get("title")
    //         .map(|t| t.to_string() + " at AllMusic")
    //         .unwrap_or(String::new());
    //     return (true, text);
    // }

    // // Parse YouTube links templates
    // if template_name == "youtube" {
    //     let params = get_params(&params, &["id", "title"]);
    //     let text = params
    //         .get("title")
    //         .map(|t| t.to_string() + " on YouTube")
    //         .unwrap_or(String::new());
    //     return (true, text);
    // }

    // // Parse Soccerway links templates
    // if template_name == "soccerway" {
    //     let params = get_params(&params, &["id", "name"]);
    //     let text = params
    //         .get("name")
    //         .map(|t| t.to_string() + " at Soccerway")
    //         .unwrap_or(String::new());
    //     return (true, text);
    // }

    // // Parse birthdate and year templates
    // if ["birth date and age",
    //     "bda", 
    //     "death date and age",
    //     "birth date",
    //     "birth date and age2"].contains(&template_name.as_str())
    // {
    //     let params = get_params(&params, &["year", "month", "day"]);
    //     let year = params["year"];
    //     let month = params["month"];
    //     let day = params["day"];

    //     let month = month
    //         .parse::<usize>()
    //         .map(|m| MONTHS[m - 1])
    //         .unwrap_or(month);

    //     return (false, format!("{month} {day}, {year}"));
    // }

    // // Parse rollover abbreviations
    // if template_name == "abbr" ||
    //     template_name == "tooltip"
    // {
    //     let params = get_params(&params, &["text", "meaning"]);
    //     let text = params["text"];
    //     let meaning = params["meaning"];
    //     return (true, format!("{} ({})", text, meaning));
    // }

    // // Parse Olympic athletes
    // if template_name.starts_with("flagioc") &&
    //     template_name.ends_with("athlete")
    // {
    //     let params = get_params(&params, &["name", "country"]);
    //     let name = params["name"];
    //     let country = params["country"];
    //     return (true, format!("{} ({})", name, country));
    // }

    // // Handle olympic flag templates
    // if template_name.starts_with("flagioc") &&
    //     !template_name.ends_with("athlete")
    // {
    //     let country = IOC::try_from(unnamed_params.get(0)?.to_lowercase().as_str());
    //     if let Ok(country) = country {
    //         let country = country.to_country();
    //         return (false, country.iso_short_name().to_string());
    //     }
    // }

    // // Parse color boxes
    // if template_name == "color box" {
    //     let params = get_params(&params, &["color", "text"]);
    //     let text = params.get("text").unwrap_or(&"");
    //     return (true, text.to_string());
    // }

    // // Parse Japanese translation helpers
    // if template_name == "nihongo" {
    //     let params = get_params(&params, &["english", "kanji", "romaji", "extra1", "extra2"]);
    //     let params: HashMap<_, _> = params
    //         .into_iter()
    //         .filter(|(_, s)| !s.is_empty())
    //         .collect();

    //     let english = params.get("english");
    //     let kanji = params["kanji"];
    //     let romaji = params.get("romaji");
    //     let extra1 = params.get("extra1");
    //     let extra2 = params.get("extra2");

    //     // Determine main display of text
    //     let formatted_text = if let Some(english) = english {
    //         english
    //     }
    //     else if let Some(romaji) = romaji {
    //         romaji
    //     }
    //     else {
    //         ""
    //     };
    //     let mut formatted_text = formatted_text.to_string() + "(" + kanji;

    //     // Add romaji in parens if english wasn't present
    //     if english.is_none() {
    //         if let Some(romaji) = romaji {
    //             formatted_text += ", ";
    //             formatted_text += romaji;
    //         }
    //     }

    //     // Add extra1 in parens if present
    //     if let Some(extra1) = extra1 {
    //         formatted_text += ", ";
    //         formatted_text += extra1;
    //     }

    //     // Terminate parens and add extra2 if present
    //     formatted_text += ")";
    //     if let Some(extra2) = extra2 {
    //         formatted_text += " ";
    //         formatted_text += extra2;
    //     }

    //     return (true, formatted_text);
    // }

    // // Parse Chinese translation helpers
    // if template_name == "zh" {
    //     let params = get_params(&params, &[]);

    //     let order = [
    //         "t",
    //         "s",
    //         "c",
    //         "p",
    //         "hp",
    //         "tp",
    //         "w",
    //         "j",
    //         "cy",
    //         "sl",
    //         "poj",
    //         "zhu",
    //         "l"
    //     ];
    //     let mut vals = Vec::new();
    //     for name in order {
    //         params.get(name).map(|&t| vals.push(t));
    //     }

    //     return (true, vals.join("; "))
    // }

    // // Parse US house of representatives templates
    // if template_name == "ushr" {
    //     let params = get_params(&params, &["state", "number"]);
    //     let state = params["state"];
    //     let number = params["number"];

    //     let number = if number == "AL" {
    //         "at-large".to_string()
    //     }
    //     else {
    //         number.to_string() + "th"
    //     };

    //     return (true, format!("{}'s {} congressional district", state, number));
    // }

    // // Parse height data
    // if template_name == "height" {
    //     let units = params
    //         .iter()
    //         .map(|p| p.split('=').map(|s| s.trim()).collect::<Vec<_>>())
    //         .map(|v| (v.get(0)?, v.get(1)?))
    //         .filter(|(name, _)| {
    //             ![
    //                 "precision",
    //                 "frac",
    //                 "abbr",
    //                 "wiki",
    //                 "out"
    //             ].contains(&name)
    //         })
    //         .map(|(name, val)| format!("{} {}", val, name))
    //         .collect::<Vec<_>>();
    //     return (false, units.join(" "));
    // }

    // // Parse font templates
    // if template_name == "font" {
    //     let params = get_params(&params, &["text"]);
    //     return (true, params["text"].to_string());
    // }



    // Handle templates that can always be totally removed
    let remove = REMOVE_TEMPLATES
        .iter()
        .any(|&s| input.to_lowercase().starts_with(s));
    if remove {
        return Some(String::new());
    }

    Some(String::from("{{") + &input + "}}")
}

// Get the template parameters. 
fn get_params<'a, 'b>(
    in_params: &'a [&'a str]
) -> HashMap<String, &'a str> 
where
    'b: 'a
{
    let mut named_params = HashMap::new();
    let mut unnamed_params = Vec::new();
    for param in in_params {
        // Divide param term by =
        let param_pieces: Vec<_> = param.split('=').map(|s| s.trim()).collect();

        // No = means unnamed param
        if param_pieces.len() == 1 {
            unnamed_params.push(param_pieces[0]);
        }

        // Record named param
        else {
            let tag_name = param_pieces[0];
            named_params.insert(tag_name.to_string(), param_pieces[1]);
        }
    }

    let mut tag_number = 1;
    for unnamed_param in unnamed_params {
        loop {
            let key = tag_number.to_string();
            if named_params.contains_key(key.as_str()) {
                tag_number += 1;
            }
            else {
                break
            }
        }

        let key = tag_number.to_string();
        named_params.insert(key, unnamed_param);
    }

    named_params
}

// Renamed unnamed params with the given names
fn rename_params<'a>(
    mut in_params: HashMap<String, &'a str>, 
    param_names: &[&str]
) -> HashMap<String, &'a str> {
    let counter = 1;
    for param_name in param_names {
        let key = counter.to_string();
        if let Some(val) = in_params.remove(key.as_str()) {
            in_params.insert(param_name.to_string(), val);
        }
    }
    return in_params;
}