use html_escape::decode_html_entities;
use std::str::from_utf8;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_until};
use nom::combinator::{map, opt, peek};
use nom::multi::{many0, many_till};
use nom::sequence::{delimited, preceded, tuple, terminated};

const REMOVE_TEMPLATES: &[&str] = &[
    "use",
    "good article",
    "infobox"
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

// Parse template items
fn template_parser(input: &str) -> IResult<&str, String> {
    // Get the contents of the template and filter unneeded ones
    map(
        preceded(
            tag("{{"),
            template_parser_worker
        ),
        filter_templates
    )(input)
}

// Returns the contents of the template
fn template_parser_worker(input: &str) -> IResult<&str, String> {
    map(
        // Grab text and sub templates until the end of this template
        many_till(
            map(
                tuple((
                    // Grab until the start of a new template or end of current one
                    many_till(
                        take(1u8),
                        peek(
                            alt((tag("{{"), tag("}}")))
                        )
                    ),

                    // See if next item is a template
                    opt(template_parser)
                )),
                |((strings, _), opt_brace)| { 
                    let brace_sub = opt_brace.unwrap_or(String::new());
                    strings.join("") + &brace_sub
                }
            ),
            tag("}}")
        ),
        |(strings, _)| strings.join("")
    )(input)
}

fn filter_templates(input: String) -> String {
    // Handle templates that can be removed
    let remove = REMOVE_TEMPLATES
        .iter()
        .any(|&s| input.to_lowercase().starts_with(s));
    if remove {
        return String::new();
    }

    return input;
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