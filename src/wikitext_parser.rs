use html_escape::decode_html_entities;
use std::str::from_utf8;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_until};
use nom::combinator::{map, opt, peek, map_parser};
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
                general_formatting_parser,
                map(take(1u8), |c: &str| c.to_owned())
            ))
        ),
        |strings| { strings.join("") }
    )(input)
}

fn template_parser(input: &str) -> IResult<&str, String> {
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
                        take(1u8),
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

fn general_formatting_parser(input: &str) -> IResult<&str, String> {
    let helper = |worker: fn(_) -> _| {
        alt((
            map_parser(worker, article_parser),
            map(worker, |s| s.to_owned())
        ))
    };

    alt((
        helper(quote_parser_worker),
        helper(link_parser_worker)
    ))(input)
}

fn quote_parser_worker(input: &str) -> IResult<&str, &str> {
    let helper = |delimiter| {
        delimited(
            tag(delimiter), 
            take_until(delimiter), 
            tag(delimiter)
        )
    };

    alt((
        helper("'''''"),
        helper("'''"),
        helper("''")
    ))(input)
}

fn link_parser_worker(input: &str) -> IResult<&str, &str> {
    map(
        delimited(
            tag("[["),
            take_until("]]"),
            tag("]]")
        ),
        |s: &str| s.split("|").last().unwrap()
    )(input)
}