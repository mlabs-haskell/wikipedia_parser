use html_escape::decode_html_entities;
use regex::Regex;
use std::collections::LinkedList;
use std::str::from_utf8;

use nom::{IResult, Parser, InputLength};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, tag_no_case};
use nom::character::complete::{none_of, anychar};
use nom::combinator::{map, peek, eof, fail};
use nom::error::ParseError;
use nom::multi::{many0, many_till, many1};
use nom::sequence::{delimited, preceded, terminated, tuple};

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
    "inflation"
];

const REMOVE_SECTIONS: &[&str] = &[
    "see also",
    "notes",
    "references",
    "external links",
    "footnotes"
];

// Take a given wikitext-formatted string and extract the useful text
pub fn extract_text(input: &[u8]) -> String {
    let input = from_utf8(input).unwrap();

    // Convert html codes to their proper characters
    let input = decode_html_entities(input).to_string();
    let input = input.replace("&ndash;", "\u{2013}");
    let input = input.replace("&nbsp;", "\u{00a0}");

    // Use nom to parse the important information from the article
    let output = article_parser(input.as_str());

    // Remove all double (or more) carriage returns
    let re = Regex::new(r"\n\n+").unwrap();
    let output = re.replace_all(&output, "\n");
    let output = output.to_string();

    output
}

// Nom parser that allows us to extract needed text while knowing the article structure
fn article_parser(input: &str) -> String {
    let result = map(
        many0(general_content_parser),
        |strings| { strings.join("") }
    )(input);

    // This is safe because the above parser will always succeed
    let (_, output) = result.unwrap();

    // Perform some final cleanup
    let output = output.trim().to_owned();
    let output = output.replace("(pronounced )", "");

    output
}

// If next item is special, parse it. Otherwise, move forward one char
fn general_content_parser(input: &str) -> IResult<&str, String> {
    alt((
        table_parser,
        template_parser, 
        section_parser,
        link_parser,
        ref_parser,
        div_parser,
        quote_parser,
        comment_parser,
        list_parser,
        html_code_parser,
        map(anychar, |c| c.to_string())
    ))(input)
}

// Remove list formatting from list items
fn list_parser(input: &str) -> IResult<&str, String> {
    map(
        look_ahead_delimited(
            tuple((
                tag("\n"),
                many1(
                    alt((
                        tag("*"),
                        tag(":")
                    ))
                )
            )), 
            general_content_parser, 
            peek(tag("\n"))
        ),
        |strings| "\n".to_owned() + &strings.join("")
    )(input)
}

// Remove unneeded sections
fn section_parser(input: &str) -> IResult<&str, String> {
    let mut header_helper = map(
        delimited(
            tag("=="), 
            many1(
                none_of("=")
            ),
            tuple((
                tag("=="),
                none_of("=")
            ))
        ),
        |v| {
            let s: String = v.iter().collect();
            s.trim().to_lowercase()
        }
    );

    let (new_input, header) = header_helper(input)?;
    if REMOVE_SECTIONS.iter().any(|r| r == &header) {
        map(
            many_till(
                alt((
                    comment_parser,
                    map(
                        anychar, 
                        |c| c.to_string()
                    )
                )), 
                alt((
                    peek(header_helper),
                    map(eof, |s: &str| s.to_string())
                ))
            ), 
            |_| String::new()
        )(new_input)
    }
    else {
        fail(input)
    }
}

// For now, just remove tables
// TODO: We may want to grab text from tables
fn table_parser(input: &str) -> IResult<&str, String> {
    map(
        alt((
            look_ahead_delimited(
                alt((
                    // Standard start for a table
                    tag("\n{|"),

                    // Templates that can start tables
                    tag_no_case("{{Awards table"),
                    tag_no_case("{{Certification Table Top")
                )),
                general_content_parser,
                tag("|}")
            ),

            // Some tables have odd headers and footers
            look_ahead_delimited(
                tag_no_case("{{Certification Table Top"),
                general_content_parser,
                tag_no_case("{{Certification Table Bottom}}")
            ),
            look_ahead_delimited(
                tag_no_case("{{Refbegin"),
                general_content_parser,
                tag_no_case("{{Refend}}")
            )
        )),
        |_| String::new()
    )(input)
}

// Parse refs and get rid of them
fn ref_parser(input: &str) -> IResult<&str, String> {
    // Take the opening tag
    let (input, tag_attrs) = delimited(
        tag("<ref"),
        take_until(">"),
        tag(">")
    )(input)?;

    // If the tag is empty, we have consumed it and we are done
    if tag_attrs.ends_with("/") {
        Ok((input, String::new()))
    }

    // Otherwise, consume until the end tag
    else {
        let (input, _) = terminated(
            take_until("</ref>"),
            tag("</ref>")
        )(input)?;

        Ok((input, String::new()))
    }
}

// Parse divs and get rid of them
fn div_parser(input: &str) -> IResult<&str, String> {
    map(
        alt((
            delimited(
                tag("<div"),
                take_until(">"),
                tag(">")
            ),
            tag("</div>")
        )),
        |_| String::new()
    )(input)
}

fn html_code_parser(input: &str) -> IResult<&str, String> {
    let helper = |html_tag| {
        look_ahead_delimited(
            tuple((
                tag("<"), 
                tag(html_tag), 
                many0(none_of(">")),
                tag(">")
            )), 
            alt((
                html_code_parser,
                map(
                    anychar, 
                    |c| c.to_string()
                )
            )), 
            delimited(
                tag("</"), 
                tag(html_tag), 
                tag(">")
            )
        )
    };

    alt((
        map(
            alt((
                helper("big"),
                helper("sub"),
                helper("sup"),
                helper("span")
            )),
            |strings| strings.concat()
        ),
        map(
            alt((
                helper("imagemap"),
                helper("gallery")
            )),
            |_| String::new()
        )
    ))(input)
}

// Get the contents of the template and filter unneeded ones
fn template_parser(input: &str) -> IResult<&str, String> {
    map(
        look_ahead_delimited(
            tag("{{"),
            alt((
                template_parser,
                map(anychar, |c| c.to_string())
            )), 
            tag("}}")
        ),
        |input| {
            let input = input.concat();
            let input = article_parser(&input);
            filter_templates(input)
        }
    )(input)
}

// TODO: Break this function into smaller pieces, or even move it into its own module
fn filter_templates(input: String) -> String {
    // Handle templates that can always be totally removed
    let remove = REMOVE_TEMPLATES
        .iter()
        .any(|&s| input.to_lowercase().starts_with(s));
    if remove {
        return String::new();
    }

    // Handle simple map cases
    let parts: Vec<_> = input.split('|').collect();
    let num_parts = parts.len();
    match parts[0].to_lowercase().as_str() {
        "sic" => return article_parser(parts[num_parts - 1]),
        "vr" => return article_parser(parts[num_parts - 1]),
        "script" => return article_parser(parts[num_parts - 1]),
        "midsize" => return article_parser(parts[num_parts - 1]),
        "'\"" => return article_parser(parts[num_parts - 1]),
        "convert" => return parts[1].to_string() + " " + parts[2],
        "sclass" => return format!("{}-class {}", parts[1].trim(), parts[2].trim()),
        "uss" => {
            if parts.len() == 2 {
                return format!("USS {}", parts[1].trim());
            }
            else if parts.len() == 3 {
                return format!("USS {} ({})", parts[1].trim(), parts[2].trim());
            }
            else {
                ()
            }
        },
        _ => ()
    }
    if parts[0].to_lowercase().starts_with("angbr") {
        return article_parser(parts[num_parts - 1])
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
        return article_parser(&output);
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
        return article_parser(&output);
    }

    return String::from("{{") + &input + "}}";
}

// Handle the command codes for bolds and italics 
fn quote_parser(input: &str) -> IResult<&str, String> {
    let helper = |delimiter| {
        map(
            look_ahead_delimited(
                tag(delimiter), 
                general_content_parser, 
                tag(delimiter)
            ),
            |strings| strings.concat()
        )
    };

    alt((
        helper("'''''"),
        helper("'''"),
        helper("''")
    ))(input)
}

// Handle the command codes for links to other articles and to images
fn link_parser(input: &str) -> IResult<&str, String> {
    map(
        look_ahead_delimited(
            tag("[["), 
            alt((
                link_parser, 
                map(anychar, |c| c.to_string())
            )), 
            tag("]]")
        ),
        |v: Vec<String>| {
            let s = v.concat();
            let s = article_parser(&s);
            if s.starts_with("File:") {
                String::new()
            }
            else {
                s.split("|").last().unwrap().to_owned()
            }
        }
    )(input)
}

// Remove comments
fn comment_parser(input: &str) -> IResult<&str, String> {
    let (new_input, _) = delimited(
        tag("<!--"),
        take_until("-->"),
        tag("-->")
    )(input)?;

    Ok((new_input, String::new()))
}

// Helper function that applies first parser, then repeatedly calls second until third succeeds
fn look_ahead_delimited<I, O1, O, O3, E, P1, P2, P3>(
    start: P1,
    body: P2,
    end: P3
) -> impl FnMut(I) -> IResult<I, Vec<O>, E> 
where
    P1: Parser<I, O1, E>,
    P2: Parser<I, O, E>,
    P3: Parser<I, O3, E>,
    E: ParseError<I>,
    I: Clone + InputLength
{
    map(
        preceded(
            start, 
            many_till(
                body, 
                end
            )
        ),
        |(o1, _)| o1
    )
}