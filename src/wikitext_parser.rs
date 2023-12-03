use html_escape::decode_html_entities;
use std::str::from_utf8;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_until};
use nom::combinator::map;
use nom::multi::{many0, many_till};
use nom::sequence::{delimited, preceded, terminated};

const REMOVE_TEMPLATES: &[&str] = &[
    "use",
    "good article",
    "infobox",
    "hidden",
    "efn",
    "see also"
];

// Take a given wikitext-formatted string and extract the useful text
pub fn extract_text(input: &[u8]) -> Vec<u8> {
    let input = from_utf8(input).unwrap();

    // Convert html codes to their proper characters
    let decoded_html = decode_html_entities(input);

    let (_, parsed) = article_parser(decoded_html.as_ref()).unwrap();
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
        ref_parser,
        template_parser, 
        quote_parser,
        link_parser,
        comment_parser,
        map(take(1u8), |c: &str| c.to_owned())
    ))(input)
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
        preceded(
            tag("{{"),
            map(
                many_till(
                    general_content_parser, 
                    tag("}}")
                ),
                |(strings, _)| strings.join("")
            )
        ),
        filter_templates
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
        preceded(
            tag(delimiter),
            map(
                many_till(
                    general_content_parser,
                    tag(delimiter)
                ),
                |(strings, _)| strings.join("")
            )
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
        preceded(
            tag("[["),
            map(
                many_till(
                    general_content_parser,
                    tag("]]")
                ),
                |(strings, _)| strings.join("")
            )
        ),
        |s: String| s.split("|").last().unwrap().to_owned()
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