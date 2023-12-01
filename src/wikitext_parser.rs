use html_escape::decode_html_entities;
use std::str::from_utf8;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_until};
use nom::combinator::{map, opt, peek};
use nom::multi::{many0, many_till};
use nom::sequence::{delimited, preceded, tuple};

const REMOVE_TEMPLATES: &[&str] = &[
    "use",
    "good article",
    "infobox"
];

pub fn extract_text(input: &[u8]) -> Vec<u8> {
    let input = from_utf8(input).unwrap();

    // Convert html codes to their proper characters
    let decoded_html = decode_html_entities(input);

    let (_, parsed) = article_parser(decoded_html.as_ref()).unwrap();
    parsed.into_bytes()
}

fn article_parser(input: &str) -> IResult<&str, String> {
    map(
        many0(
            alt((
                template_parser, 
                quote_parser,
                bracket_parser,
                map(take(1 as u8), |c: &str| c.to_owned())
            ))
        ),
        |strings| { strings.join("") }
    )(input)
}

fn template_parser(input: &str) -> IResult<&str, String> {
    // TODO: This is where the transformation goes
    map(
        preceded(
            tag("{{"),
            inner_template_parser
        ),
        filter_templates
    )(input)
}

fn inner_template_parser(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            map(
                tuple((
                    many_till(
                        take(1 as u8),
                        peek(
                            alt((tag("{{"), tag("}}")))
                        )
                    ),
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

fn quote_parser(input: &str) -> IResult<&str, String> {
    alt((
        quote_helper("'''''"),
        quote_helper("'''"),
        quote_helper("''")
    ))
(input)
}

fn quote_helper<'a>(delimiter: &'static str) 
    -> impl FnMut(&'a str) -> IResult<&'a str, String> 
{
    map(
        preceded(
            tag(delimiter), 
            many_till(
                take(1 as u8), 
                tag(delimiter)
            ), 
        ),
        |(strings, _)| strings.join("")
    )
}

fn bracket_parser(input: &str) -> IResult<&str, String> {
    map(
        delimited(
            tag("[["),
            take_until("]]"),
            tag("]]")
        ),
        |s: &str| s.split("|").last().unwrap().to_owned()
    )(input)
}