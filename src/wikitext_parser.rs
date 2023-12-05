use html_escape::decode_html_entities;
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
    "refbegin",
    "refend",
    "cite",
    "short description",
    "about",
    "pp-protected",
    "technical reasons"
];

const REMOVE_SECTIONS: &[&str] = &[
    "see also",
    "notes",
    "references",
    "external links"
];

// Take a given wikitext-formatted string and extract the useful text
pub fn extract_text(input: &[u8]) -> Vec<u8> {
    let input = from_utf8(input).unwrap();

    // Convert html codes to their proper characters
    let input = decode_html_entities(input).to_string();
    let input = input.replace("&ndash;", "-");
    let input = input.replace("&nbsp;", " ");

    // Use nom to parse the important information from the article
    let (_, parsed) = article_parser(input.as_str()).unwrap();

    // Perform some final cleanup
    let parsed = parsed.trim().to_owned();

    parsed.into_bytes()
}

// Nom parser that allows us to extract needed text while knowing the article structure
fn article_parser(input: &str) -> IResult<&str, String> {
    map(
        many0(general_content_parser),
        |strings| { strings.join("") }
    )(input)
}

// If next item is special, parse it. Otherwise, move forward one char
fn general_content_parser(input: &str) -> IResult<&str, String> {
    alt((
        table_parser,
        ref_parser,
        template_parser, 
        quote_parser,
        link_parser,
        comment_parser,
        section_parser,
        list_parser,
        map(anychar, |c| c.to_string())
    ))(input)
}

fn list_parser(input: &str) -> IResult<&str, String> {
    map(
        look_ahead_delimited(
            tuple((
                tag("\n"),
                many1(tag("*"))
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
            tag("==")
        ),
        |v| {
            let s: String = v.iter().collect();
            s.trim().to_lowercase()
        }
    );

    let (input, header) = header_helper(input)?;
    if REMOVE_SECTIONS.iter().any(|r| r == &header) {
        map(
            many_till(
                anychar, 
                alt((
                    peek(header_helper),
                    map(eof, |s: &str| s.to_string())
                ))
            ), 
            |_| String::new()
        )(input)
    }
    else {
        fail(input)
    }
}

// For now, just remove tables
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
        |strings| filter_templates(strings.concat())
    )(input)
}

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
        "sic" => return article_parser(parts[num_parts - 1]).unwrap().1,
        _ => ()
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
        return article_parser(&output).unwrap().1;
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
            general_content_parser, 
            tag("]]")
        ),
        |v: Vec<String>| {
            let s = v.concat();
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
    let (input, _) = delimited(
        tag("<!--"),
        take_until("-->"),
        tag("-->")
    )(input)?;

    Ok((input, String::new()))
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