use html_escape::decode_html_entities;
use std::str::from_utf8;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::combinator::{map, opt, peek};
use nom::multi::{many0, many_till};
use nom::sequence::{preceded, tuple};

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
                brace_parser, 
                map(take(1 as u8), |c: &str| c.to_owned())
            ))
        ),
        |strings| { strings.join("") }
    )(input)
}

fn brace_parser(input: &str) -> IResult<&str, String> {
    // TODO: This is where the transformation goes
    preceded(
        tag("{{"),
        inner_brace_parser
    )(input)
}

fn inner_brace_parser(input: &str) -> IResult<&str, String> {
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
                    opt(brace_parser)
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