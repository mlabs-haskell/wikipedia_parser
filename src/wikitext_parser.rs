use html_escape::decode_html_entities;
use regex::Regex;
use std::str::from_utf8;

use nom::{IResult, Parser, InputLength};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, tag_no_case};
use nom::character::complete::{none_of, anychar, one_of};
use nom::combinator::{map, peek, eof, fail};
use nom::error::ParseError;
use nom::multi::{many0, many_till, many1};
use nom::sequence::{delimited, preceded, terminated, tuple};

use crate::template_transformers::filter_templates;

const REMOVE_SECTIONS: &[&str] = &[
    "see also",
    "notes",
    "references",
    "external links",
    "footnotes",
    "further reading",
    "gallery",
    "bibliography" ,
    "explanatory notes",
    "citations",
    "general bibliography",
    "notes and references"
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
                    one_of("*:;")
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
            tag("\n=="), 
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
                helper("span"),
                helper("blockquote"),
                helper("abbr"),
                helper("poem")
            )),
            |strings| {
                let s = strings.concat();
                article_parser(&s)
            }
        ),
        map(
            alt((
                helper("imagemap"),
                helper("gallery"),
                helper("math"),
                helper("score")
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
            let (needs_parsing, input) = filter_templates(input);
            if needs_parsing {
                article_parser(&input)
            }
            else {
                input
            }
        }
    )(input)
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