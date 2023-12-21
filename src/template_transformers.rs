use lazy_static::lazy_static;

use keshvar::IOC;
use std::collections::{LinkedList, HashMap, HashSet, BTreeMap};

const REMOVE_TEMPLATES: &[&str] = &[
    // "#tag",
    // "0",
    "4teambracket-tennis3",
    "16teambracket-compact-tennis3",
    "about",
    // "according to whom",
    // "additional citation needed",
    "advert",
    "afb game box",
    // "agriculture",
    "ahnentafel",
    "aircraft specs",
    // "alabama",
    // "album chart",
    "album ratings",
    // "algeria",
    // "ambiguous",
    "american football roster",
    // "anarchism",
    "anchor",
    // "ancient greek religion",
    // "anthropology",
    // "apollo",
    "authority control",
    "automatic taxobox",
    // "awards table",
    "bar box",
    "bar percent",
    "basho",
    // "basic forms of government",
    "basketballbox",
    "better source needed",
    "blp",
    // "broader",
    // "by whom",
    "canadian election result",
    // "canadian party colour",
    "canelec",
    "cascite",
    "cbb roster",
    "cbb schedule",
    "cbb yearly record",
    // "cbignore",
    // "certification cite ref",
    // "certification table",
    "cfb schedule",
    "cfb yearly record",
    // "chset-cell1",
    // "charmap",
    // "chart",
    "chembox",
    "citation",
    "cite",
    "clade",
    "clarify",
    "cleanup",
    "clear",
    "cn",
    "col-",
    "colbegin",
    "colend",
    "commons category",
    // "contains special characters",
    "coord missing",
    // "css",
    // "cyber",
    "dab",
    // "date table sorting",
    "dead link",
    // "decrease",
    // "defaultsort",
    // "detailslink",
    "disambig",
    "disambiguation",
    "distinguish",
    "div col",
    "draw key",
    "drugbox",
    "dts", // If this turns out to be used outside of a table, we'll need to handle it
    // "dubious",
    "dynamic list",
    // "economic",
    "efn",
    "efs player",
    "election box",
    "election results",
    // "elucidate",
    "empty section",
    "engvarb",
    "episode list", // This is a table, but we can probably extract information from it
    "episode table", // Same as above
    // "esotericism",
    // "etymology",
    // "excerpt",
    "expand",
    "extended football squad player",
    "external media",
    "fact",
    "failed verification",
    "family name hatnote",
    // "fbaicon",
    "fdacite",
    // "featured article",
    "fhgoal",
    // "flagcountry",
    // "flagdeco",
    "flagicon",
    "football box",
    "footballbox",
    "for multi",
    // "formatnum",
    // "france metadata wikidata",
    "fs player",
    // "full citation needed",
    "further",
    "gallery",
    "gbmapping",
    "geodis",
    "geogroup",
    "given name",
    // "globalize",
    "good article",
    "goal",
    // "greek myth",
    // "harvid",
    // "harvnb",
    "hatnote",
    // "hermeticism",
    // "hidden",
    "historical populations",
    "hndis",
    "hs listed building",
    // "image",
    // "imdb",
    "in lang",
    "incomplete list",
    "infobox",
    "ipa", // TODO: We can probably do something with IPA pronunciations
    "italic title",
    "jctbtm",
    "jcttop",
    // "largest cities",
    // "latin letter",
    // "leagueicon",
    "legend",
    // "letter other reps",
    "listen",
    "location map",
    "lomp",
    // "london gazette",
    "main",
    // "maplink",
    // "marriage", // Only meant to appear in infoboxes
    "math",
    "medal", 
    // "medical",
    "more citations needed",
    "more footnotes",
    "multiple image",
    "multiple issues",
    "music ratings",
    // "music",
    "nat fs g player",
    "nat fs player",
    "national football squad player",
    // "nhle",
    "nhrp",
    "no footnotes",
    "notability",
    "note",
    "notelist",
    // "nts",
    // "official website",
    "one source",
    "oneleg",
    // "open access",
    "orphan",
    "other people",
    "other places",
    "other ships",
    "other uses",
    // "page needed",
    "party color",
    // "party shading",
    "party stripe",
    // "pb",
    // "pengoal",
    // "performance",
    // "plainlist",
    // "political",
    "portal",
    "portuguese name",
    // "pp-protected",
    // "pp",
    "presfoot",
    "preshead",
    "presrow",
    "primary sources",
    "redirect",
    "refimprove",
    "reflist",
    "refn",
    // "relevance",
    "respell", // This is another IPA-related item
    "rp",
    "rugbybox",
    // "s-aft",
    // "s-bef",
    // "s-end",
    "s-start",
    // "s-ttl",
    "see",
    "see also",
    "sfn",
    "shipwreck list",
    "short description",
    "short pages monitor",
    // "shy",
    // "single chart",
    // "singlechart",
    "sort",
    // "spaceflight",
    "speciesbox",
    // "specify",
    "stack",
    "stv election box",
    // "subon",
    // "succession box",
    "sumo record",
    // "sup",
    "surname",
    // "table",
    "taxobox",
    // "taxonbar",
    // "technical reasons",
    "tennis events",
    "toc",
    "track listing", // Something can probably be done with this one
    "tracklist",
    "twoleg",
    // "undue weight section",
    "unreferenced",
    // "unreliable source",
    "update",
    // "url",
    "us census population",
    // "usa",
    "use",
    // "vague",
    "video game reviews",
    "weather box",
    "webarchive",
    "when",
    "wide image",
    "who",
    "wikidata",
    // "wikisource",
    "wiktionary",
    "x",
    "yel"
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
            ("spnd", " - "),
            ("spaced ndash", " - "),
            ("ndash", "-"),
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
            ("ya", ""),
            ("=", "="),
            ("pi", "π"),
            ("-", "")
        ].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    };

    static ref REPLACE_LAST: HashSet<String> = {
        [
            // "avoid wrap",
            // "angbr",
            // "australian party style",
            // "center",
            // "crossref",
            // "crossreference",
            // "fb",
            // "fbw",
            // "flagg",
            // "flagu",
            // "flatlist",
            "m+j",
            // "keypress",
            "lang",
            // "linktext",
            // "medalsport",
            // "midsize",
            // "née",
            // "notatypo",
            // "oclc",
            // "pslink",
            // "script",
            // "section link",
            // "sic",
            "transl",
            "transliteration",
            // "vr",
        ].iter().map(|s| s.to_string()).collect()
    };

    static ref REPLACE_FIRST: HashSet<String> = {
        [
            "cast listing",
            "columns-list",
            // "cr",
            "ct",
            // "esc",
            "flag",
            "ill",
            "interlanguage link",
            "interlanguage link multi",
            "mesh",
            // "oldstyledateny",
            "stn",
            "station",
            "sup"
        ].iter().map(|s| s.to_string()).collect()
    };

    static ref MERGE_WITH_SPACES: HashSet<String> = {
        [
            "airport codes",
            "au",
            "hlist",
            "linktext",
            "minor planet",
            "mp",
            "nflplayer",
            "post-nominals",
            "postnominals",
            "ship"
        ].iter().map(|s| s.to_string()).collect()
    };

    static ref CONCATENATE: HashSet<String> = {
        [
            "not a typo",
            "typo",
            "proper name",
            "chem name",
            "as written",
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
    // Get the template name and its params
    let parts: Vec<_> = input
        .split('|')
        .map(|s| s.trim())
        .collect();
    let template_name: Vec<_> = parts[0]
        .split(' ')
        .filter(|s| !s.is_empty())
        .collect();
    let template_name = template_name.join(" ").to_lowercase().trim().to_string();
    let params = get_params(&parts[1..]);

    // Handle templates that can be simply mapped to a constant
    let mapping = MAPPERS.get(&template_name);
    if mapping.is_some() {
        return mapping.cloned();
    }

    // Handle any template that should be replaced with its last parameter
    let replace = REPLACE_LAST.contains(&template_name);
    if replace {
        return params
            .iter()
            .filter_map(|(k, v)| {
                let k = k.parse::<usize>().ok()?;
                Some((k, v))
            })
            .max_by_key(|&(k, _)| k)
            .map(|(_, v)| v.to_string());
    }

    // Handle any template that should be replaced with its first parameter
    let replace = REPLACE_FIRST.contains(&template_name);
    if replace {
        return params
            .iter()
            .filter_map(|(k, v)| {
                let k = k.parse::<usize>().ok()?;
                Some((k, v))
            })
            .min_by_key(|&(k, _)| k)
            .map(|(_, v)| v.to_string());
    }

    // Handle any template where the unnamed params can be joined with spaces
    let replace = MERGE_WITH_SPACES.contains(&template_name);
    if replace {
        let params: BTreeMap<_, _> = params
            .iter()
            .filter_map(|(k, &v)| {
                let k = k.parse::<usize>().ok()?;
                Some((k, v))
            })
            .collect();
        let params: Vec<_> = params.values().map(|&v| v).collect();
        return Some(params.join(" "));
    }

    // Handle any template where the unnamed params can just be concatenated
    let replace = CONCATENATE.contains(&template_name);
    if replace {
        let params: BTreeMap<_, _> = params
            .iter()
            .filter_map(|(k, &v)| {
                let k = k.parse::<usize>().ok()?;
                Some((k, v))
            })
            .collect();
        let params: Vec<_> = params.values().map(|&v| v).collect();
        return Some(params.concat());
    }

    // Handle simple parsing cases
    match template_name.as_str() {
        // "sclass" => return (false, format!("{}-class {}", unnamed_params.get(0)?, unnamed_params.get(1)?)),
        "uss" | "hms" | "hmnzs" => {
            let s = if parts.len() == 2 {
                format!("{} {}", parts[0], parts[1])
            }
            else {
                format!("{} {} ({})", parts.get(0)?, parts.get(1)?, parts.get(2)?)
            };
            return Some(s);
        },
        // "see below" => return (true, format!("(see {})", unnamed_params.get(0)?)),
        "c." | "circa" => {
            if let Some(date1) = params.get("1") {
                if let Some(date2) = params.get("2") {
                    return Some(format!("{} {}-{}", parts[0], date1, date2));
                }
                else {
                    return Some(format!("{} {}", parts[0], date1));
                }
            }
            else {
                return Some(parts[0].to_string());
            }
        },
        "frac" | "fraction" => {
            match params.len() {
                0 => return Some("/".to_string()),
                1 => return Some(format!("1/{}", params.get("1")?)),
                2 => return Some(format!("{}/{}", params.get("1")?, params.get("2")?)),
                3 => return Some(format!(
                    "{} {}/{}", 
                    params.get("1")?, 
                    params.get("2")?, 
                    params.get("3")?)
                ),
                _ => return None
            };
        },
        "nee" => {
            let name = params
                .get("1")
                .map(|v| " ".to_string() + v)
                .unwrap_or(String::new());
            return Some("née".to_string() + &name)
        },
        "nowrap" | "mvar" => return Some(parts[1..].concat()),
        "rating" => {
            let score = params.get("1")?;
            let possible = params.get("2");
            if let Some(possible) = possible {
                return Some(format!("{score}/{possible}"));
            }
            else {
                return Some(score.to_string());
            }
        },
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
        "player" => return Some(params.get("1")?.to_string() + " " + params.get("3")?),
        "rws" | "stnlnk" => 
            return params.get("3").or(params.get("1")).map(|s| s.to_string()),
        "sclass" => {
            let classname = params.get("1")?;
            let shiptype = params.get("2")?;
            return Some(format!("{}-class {}", classname, shiptype));
        },
        "small" => return params.get("1").map(|s| s.to_string()),
        "sortname" => {
            let params = rename_params(params, &["first", "last"]);
            let first = params.get("first")?;
            return Some(
                params
                    .get("last")
                    .map(|l| first.to_string() + " " + l)
                    .unwrap_or(first.to_string())
            );
        },
        "translation" => {
            let mut ret_val = "transl.".to_string();
            if let Some(meaning1) = params.get("1") {
                ret_val = format!("{} {}", ret_val, meaning1);
            }
            if let Some(meaning2) = params.get("2") {
                ret_val = format!("{} - transl. {}", ret_val, meaning2);
            }
            return Some(ret_val);
        },
        // "bce" | "ce" => return (true, unnamed_params.get(0)?.to_string() + " " + parts.get(0)?),
        // "ietf rfc" => return (false, format!("RFC {}", unnamed_params.join(", "))),
        // "mlbplayer" => return (true, unnamed_params.get(1)?.to_string()),
        // "fbu" | "fb-rt" => return (true, unnamed_params.get(1)?.to_string()),
        // "flagmedalist" => return (true, format!("{} ({})", unnamed_params.get(0)?, unnamed_params.get(1)?)),
        // "party name with colour" | "party name with color" => 
        //     return (true, unnamed_params.get(1)?.to_string()),
        // "suboff" => return (true, unnamed_params.get(0).unwrap_or(&"").to_string()),
        "val" => {
            let number = params.get("1");
            if let Some(number) = number {
                let error = params.get("2");
                if let Some(error) = error {
                    return Some(format!("{} ± {}", number, error));
                }
                else {
                    return Some(number.to_string());
                }
            }
            else {
                let exponent = params.get("e")?;
                return Some(format!("10 ^ {}", exponent));
            }
        },
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
    if template_name == "sic" {
        // Remove unnamed params, unusable ones, and empty ones
        let params: BTreeMap<_, _> = params
            .iter()
            .filter_map(|(k, &v)| Some((k.parse::<usize>().ok()?, v)))
            .filter(|(_, v)| *v != "?")
            .collect();
        let params: Vec<_> = params.values().cloned().collect();

        return Some(params.concat())
    }

    // Handle the entire "lang" family of templates
    if template_name.starts_with("lang") {
        // Remove named params
        let params: BTreeMap<_, _> = params
            .iter()
            .filter_map(|(k, &v)| Some((k.parse::<usize>().ok()?, v)))
            .collect();
        return params.values().last().map(|s| s.to_string());
    }

    // Handle quotation blocks
    if template_name == "blockquote" ||
        template_name == "quotation" ||
        template_name == "quote" ||
        template_name == "quote box" ||
        template_name == "cquote"
    {
        let params = rename_params(params, &["text", "author", "source"]);

        let text = params
            .get("text")
            .or(params.get("quote"))
            .or(params.get("quotetext"))
            .or(params.get("content"))?;
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
            text.to_string()
        }
        else {
            format!("\"{text}\"-{caption}")
        };
        return Some(output);
    }

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

    // Parse "as of" blocks
    if template_name == "as of" ||
        template_name == "asof"
    {
        let params = rename_params(params, &["year", "month", "day"]);

        let alt = params.get("alt");
        let year = params.get("year");
        let month = params
            .get("month")
            .map(|s| 
                s
                    .parse::<usize>()
                    .ok()
                    .and_then(|i| MONTHS.get(i - 1))
                    .unwrap_or(s)
                );
        let day = params.get("day");
        let since = params.get("since");
        let post = params.get("post");

        if let Some(alt) = alt {
            return Some(alt.to_string());
        }
        else {
            let mut output = if since == Some(&"y") {
                "Since ".to_string()
            }
            else {
                let as_of = if parts.get(0)?.chars().next() == Some('A') {
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

            return Some(output);
        }
    }

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

    // Handle highway junctions
    if template_name == "jct" {  
        // Remove unnamed params, unusable ones, and empty ones
        let params: BTreeMap<_, _> = params
            .iter()
            .filter_map(|(k, &v)| Some((k.parse::<usize>().ok()?, v)))
            .collect();
        let params: Vec<_> = params.values().cloned().collect();

        let pairs = params.chunks(2);
        let highways: Vec<_> = pairs
            .into_iter()
            .map(|pair| pair.join("-"))
            .collect();

        return Some(highways.join("/"))
    }

    // Parse coordinate templates
    if template_name == "coord" ||
        template_name == "coordinates" ||
        template_name == "location"
    {
        // Remove unnamed params, unusable ones, and empty ones
        let params: HashMap<_, _> = params
            .iter()
            .filter(|(k, v)| 
                k.chars().all(|c| c.is_numeric()) && !v.contains(":") && !v.is_empty())
            .map(|(k, &v)| (k.as_str(), v))
            .collect();

        // Sort by numeric label
        let mut params: Vec<_> = params.iter().map(|(&k, &v)| (k, v)).collect();
        params.sort_unstable_by_key(|(k, _)| *k);
        let params: Vec<_> = params.iter().map(|(_, v)| *v).collect();

        match params.len() {
            0 => return None,
            2 => {
                let (lat_letter, lat) = if params.get(0)?.starts_with('-') {
                    ('S', &params.get(0)?[1..])
                }
                else {
                    ('N', *params.get(0)?)
                };

                let (long_letter, long) = if params.get(1)?.starts_with('-') {
                    ('W', &params.get(0)?[1..])
                }
                else {
                    ('E', *params.get(0)?)
                };

                return Some(
                    format!(
                        "{}\u{00B0}{} {}\u{00B0}{}", 
                        lat, 
                        lat_letter, 
                        long, 
                        long_letter
                    )
                );
            },
            4 => return Some(
                format!(
                    "{}\u{00B0}{} {}\u{00B0}{}", 
                    params.get(0)?, 
                    params.get(1)?, 
                    params.get(2)?, 
                    params.get(3)?
                )
            ),
            6 => return Some(
                format!(
                    "{}\u{00B0}{}'{} {}\u{00B0}{}'{}", 
                    params.get(0)?, 
                    params.get(1)?, 
                    params.get(2)?, 
                    params.get(3)?,
                    params.get(4)?,
                    params.get(5)?
                )
            ),
            8 => return Some(
                format!(
                    "{}\u{00B0}{}'{}\"{} {}\u{00B0}{}'{}\"{}", 
                    params.get(0)?, 
                    params.get(1)?, 
                    params.get(2)?, 
                    params.get(3)?,
                    params.get(4)?,
                    params.get(5)?,
                    params.get(6)?,
                    params.get(7)?
                )
            ),
            _ => ()
        }
    }

    if template_name == "isbn" {
        // Remove unnamed params, unusable ones, and empty ones
        let params: BTreeMap<_, _> = params
            .iter()
            .filter(|(k, v)| 
                k.chars().all(|c| c.is_numeric()) && !v.is_empty())
            .map(|(k, &v)| (k.parse::<usize>().unwrap(), v))
            .collect();

        let isbns: Vec<_> = params.values().map(|&v| v).collect();
        return Some(isbns.join(", "));
    }

    // // Get sorted item from sort templates
    // if template_name == "sort" {
    //     let params = get_params(&params, &["1", "2"]);
    //     let sort_item = params.get("2").or(params.get("1"));
    //     return (true, sort_item.unwrap_or(&"").to_string());
    // }

    // Get dates
    if template_name == "start date" ||
        template_name == "start date and age" ||
        template_name == "end date"
    {
        // Get the tags we have and remove empty ones
        let params = rename_params(params, &[
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
            .and_then(|s| 
                s.parse::<usize>().ok().map(|i| MONTHS.get(i - 1).unwrap_or(s))
            );
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

        return Some(date_string)
    }

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
    if template_name == "flagathlete" {
        let params = rename_params(params, &["name", "country"]);
        let name = params.get("name")?;
        let country = params.get("country")?;
        return Some(format!("{} ({})", name, country));
    }

    // Parse inflation templates
    if template_name == "inflation" {
        let params = rename_params(params, &["index", "value", "start_year"]);
        let index = params.get("index")?;
        let value = params.get("value")?;
        let start_year = params.get("start_year")?;
        return Some(format!("{} {} ({})", index, value, start_year));
    }

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

    // Parse birthdate and year templates
    if ["birth date and age",
        "bda", 
        "death date and age",
        "birth date",
        "birth date and age2"].contains(&template_name.as_str())
    {
        let params = rename_params(params, &["year", "month", "day"]);
        let year = params.get("year")?;
        let month = params.get("month");
        let day = params.get("day");

        let prefix = month
            .map(|m| {
                let m = m
                    .parse::<usize>()
                    .ok()
                    .and_then(|m| MONTHS.get(m - 1))
                    .unwrap_or(&m);

                if let Some(d) = day {
                    format!("{m} {d}, ")
                }
                else {
                    format!("{m}, ")
                }
            })
            .unwrap_or(String::new());

        return Some(prefix + year);
    }

    // Parse rollover abbreviations
    if template_name == "abbr" ||
        template_name == "tooltip"
    {
        let params = rename_params(params, &["text", "meaning"]);
        let text = params.get("text")?;
        let meaning = params.get("meaning")?;
        return Some(format!("{} ({})", text, meaning));
    }

    if template_name.starts_with("flagioc") {
        // Display Olympic athletes
        if template_name.ends_with("athlete") || template_name.ends_with("medalist") {
            let name = params.get("1")?;
            let country = params.get("2")?;
            return Some(format!("{} ({})", name, country));
        }

        // Display Olympic countries, using IOC mappings if available
        else {
            let country = params.get("1")?;
            let ioc_country = IOC::try_from(country.to_lowercase().as_str()).ok();
            if let Some(ioc_country) = ioc_country {
                let ioc_country = ioc_country.to_country();
                return Some(ioc_country.iso_short_name().to_string());
            }
            else {
                return Some(country.to_string());
            }
        }
    }

    // // Parse color boxes
    if template_name == "color box" {
        let params = rename_params(params, &["color", "text"]);
        let text = params.get("text").unwrap_or(&"");
        return Some(text.to_string());
    }

    // // Parse Japanese translation helpers
    if template_name == "nihongo" {
        let params = rename_params(params, &["english", "kanji", "romaji", "extra1", "extra2"]);
        let params: HashMap<_, _> = params
            .into_iter()
            .filter(|(_, s)| !s.is_empty())
            .collect();

        let english = params.get("english");
        let kanji = params.get("kanji")?;
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

        return Some(formatted_text);
    }

    // Parse Chinese translation helpers
    if template_name == "zh" {
        let order = [
            "t",
            "s",
            "c",
            "p",
            "hp",
            "tp",
            "w",
            "j",
            "cy",
            "sl",
            "poj",
            "zhu",
            "l"
        ];
        let mut vals = Vec::new();
        for name in order {
            params.get(name).filter(|t| !t.is_empty()).map(|&t| vals.push(t));
        }

        return Some(vals.join("; "))
    }

    // Parse Korean translation helpers
    if template_name == "korean" {
        let params = rename_params(params, &["hangul", "hanja", "rr", "mr"]);
        let order = [
            ("hangul", "Korean"),
            ("hanja", "Hanja"),
            ("rr", "RR"),
            ("mr", "MR")
        ];
        let mut labels = Vec::new();

        for (key, text) in order {
            if let Some(val) = params.get(key) {
                labels.push(format!("{}: {}", text, val));
            }
        }

        return Some(labels.join("; "))
    }

    // Parse US house of representatives templates
    if template_name == "ushr" {
        let params = rename_params(params, &["state", "number"]);
        let state = params.get("state")?;
        let number = params.get("number")?;

        let number = if *number == "AL" {
            "at-large".to_string()
        }
        else {
            number.to_string() + "th"
        };

        return Some(format!("{}'s {} congressional district", state, number));
    }

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
    let mut counter = 1;
    for param_name in param_names {
        if in_params.contains_key(&param_name.to_string()) {
            continue;
        }

        let key = counter.to_string();
        if let Some(val) = in_params.remove(key.as_str()) {
            in_params.insert(param_name.to_string(), val);
        }
        counter += 1;
    }
    return in_params;
}