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
    "explanatory notes",
    "citations",
    "general bibliography",
    "notes and references",
    "sources"
];

const REMOVE_LINKS: &[&str] = &[
    "file:",
    "image:"
];

// Take a given wikitext-formatted string and extract the useful text
pub fn extract_text(input: &[u8]) -> String {
    let input = from_utf8(input).unwrap();

    // Convert html codes to their proper characters
    let input = decode_html_entities(input).to_string();
    let input = input.replace("&ndash;", "\u{2013}");
    let input = input.replace("&nbsp;", "\u{00a0}");
    let input = input.replace("&minus;", "-");

    // Use nom to parse the important information from the article
    let output = article_parser(input.as_str());

    // Remove all double (or more) carriage returns
    let re = Regex::new(r"\n\n+").unwrap();
    let output = re.replace_all(&output, "\n");

    // Perform some final cleanup
    let output = output.trim().to_owned();
    output.replace("(pronounced )", "")
}

// Nom parser that allows us to extract needed text while knowing the article structure
fn article_parser(input: &str) -> String {
    let result = map(
        many0(general_content_parser),
        |strings| { strings.concat() }
    )(input);

    // This is safe because the above parser will always succeed
    let (_, output) = result.unwrap();

    output
}

// If next item is special, parse it. Otherwise, move forward one char
fn general_content_parser(input: &str) -> IResult<&str, String> {  
    alt((
        table_parser,
        template_parser, 
        section_parser,
        link_parser,
        empty_tag_parser,
        quote_parser,
        comment_parser,
        list_parser,
        html_code_parser,
        no_content_tag_parser,
        map(anychar, |c| c.to_string())
    ))(input)
}

// Parsing the contents of a template when parsing a template 
// can lead to infinite recursion
fn template_contents_parser(input: &str) -> IResult<&str, String> {
    let helper = alt((
        table_parser,
        section_parser,
        link_parser,
        empty_tag_parser,
        quote_parser,
        comment_parser,
        list_parser,
        html_code_parser,
        no_content_tag_parser,
        map(anychar, |c| c.to_string())
    ));

    map(
        many0(helper),
        |strings| { strings.concat() }
    )(input)
}

// For now, just remove tables
// TODO: We may want to grab text from tables
fn table_parser(input: &str) -> IResult<&str, String> {
    map(
        alt((
            look_ahead_delimited(
                tag_no_case("{{refbegin"), 
                map(anychar, |_| String::new()), 
                tuple((
                    tag_no_case("{{refend"),
                    many0(none_of("{}")),
                    tag("}}")
                ))
            ),
            look_ahead_delimited(
                tuple((
                    tag_no_case("{{"), 
                    many1(none_of("{}")),
                    tag("}}")
                )),
                map(none_of("{}"), |_| String::new()), 
                tuple((
                    alt((
                        tag_no_case("{{end"),
                        tag_no_case("{{s-end")
                    )),
                    many0(none_of("{}")),
                    tag("}}")
                ))
            ),
            look_ahead_delimited(
                tuple((
                    tag_no_case("{{fs start"),
                    many0(none_of("{}")),
                    tag("}}")
                )),
                map(anychar, |_| String::new()), 
                tuple((
                    tag_no_case("{{fs end"),
                    many0(none_of("{}")),
                    tag("}}")
                ))
            ),
            look_ahead_delimited(
                alt((
                    // Standard start for a table
                    tag("\n{|"),

                    // Templates that can start tables
                    tag_no_case("{{Awards table"),
                    tag_no_case("{{Certification Table Top"),
                    tag_no_case("{{LegSeats3"),
                    tag_no_case("{{NRHP header"),
                    tag_no_case("{{col-begin"),
                    tag_no_case("{{HS listed building header")
                )),
                alt((
                    table_parser,
                    map(anychar, |_| String::new())
                )),
                alt((
                    terminated(
                        tag("|}"),
                        none_of("}")
                    ),
                    peek(tag("\n=="))
                ))
            ),
        )),
        |_| String::new()
    )(input)
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
            // if input.trim().to_lowercase().starts_with("stnlnk|") {
            //     println!("Raw template: {}", input);
            // }
            let (_, reparsed_input) = template_contents_parser(&input).unwrap();
            // if input.trim().to_lowercase().starts_with("stnlnk|") {
            //     println!("Reparsed template: {}", reparsed_input);
            // }
            let output = filter_templates(&reparsed_input);
            if let Some(output) = output {
                // if input.trim().to_lowercase().starts_with("stnlnk|") {
                //     println!("Final: {}", output);
                // }
                output
            }
            else {
                let skip_logging = [
                    "convert",
                    "cvt",
                    "coord",
                    "location",
                    "lang",
                    "small",
                    "nihongo",
                    "blockquote",
                    "abbr",
                    "columns-list",
                    "quote",
                    "val",
                    "player",
                    "frac"
                ];
                let skip = skip_logging
                    .iter()
                    .any(|prefix| input.to_lowercase().trim().starts_with(prefix));

                if !skip {
                    println!("Problem template: {}", input);
                }

                String::new()
            }
        }
    )(input)
}

// Remove unneeded sections
fn section_parser(input: &str) -> IResult<&str, String> {
    let mut header_helper = map(
        delimited(
            tuple((
                tag::<_, &str, _>("\n=="),
                peek(none_of("="))
            )), 
            take_until("=="),
            tuple((
                tag("=="),
                peek(none_of("="))
            ))
        ),
        |s| s.trim().to_lowercase()
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
            let s_lower = s.to_lowercase();
            if REMOVE_LINKS.iter().any(|prefix| s_lower.starts_with(prefix)) {
                String::new()
            }
            else {
                if let Some((_, out)) = s.split_once('|') {
                    out.to_string()
                }
                else {
                    s
                }
            }
        }
    )(input)
}

// Parse potentially empty tags and get rid of them
fn empty_tag_parser<'a>(input: &'a str) -> IResult<&'a str, String> {
    let helper = |tag_name| {
        move |input: &'a str| {
            let tag_opener = format!("<{}", tag_name);

            // Take the opening tag
            let (input, tag_attrs) = delimited(
                tag(tag_opener.as_str()),
                take_until(">"),
                tag(">")
            )(input)?;

            // If the tag is empty, we have consumed it and we are done
            if tag_attrs.trim().ends_with('/') {
                Ok((input, String::new()))
            }

            // Otherwise, consume until the end tag
            else {
                let tag_ender = format!("</{}>", tag_name);
                let (input, _) = terminated(
                    take_until(tag_ender.as_str()),
                    tag(tag_ender.as_str())
                )(input)?;

                Ok((input, String::new()))
            }
        }
    };

    // Allow for an early bailout
    let (input, _) = peek(tag("<"))(input)?;
    alt((
        helper("ref"),
        helper("nowiki")
    ))(input)
}

// Handle the command codes for bolds and italics 
fn quote_parser(input: &str) -> IResult<&str, String> { 
    // Allow for an early bailout
    let (input, _) = peek(tag("''"))(input)?;

    map(
        alt((
            preceded(tag("'''''"), five_quote_state),
            preceded(tag("'''"), three_quote_state),
            preceded(tag("''"), two_quote_state)
        )),
        |s| article_parser(&s)
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

// Remove list formatting from list items
fn list_parser(input: &str) -> IResult<&str, String> {
    map(
        preceded(
            tuple((
                tag("\n"),
                many1(
                    one_of("*:;#")
                )
            )), 
            many_till(
                alt((
                    html_code_parser,
                    template_parser,
                    map(anychar, |c| c.to_string())
                )),
                peek(tag("\n"))
            )
        ),
        |(strings, _)| {
            let s = "\n".to_string() + &strings.concat();
            article_parser(&s)
        }
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
                preceded(
                    many0(none_of(">")),
                    tag(">")
                )
            )
        )
    };

    // Allow for an early bailout
    let (input, _) = peek(tag("<"))(input)?;
    alt((
        map(
            alt((
                helper("small"),
                helper("big"),
                helper("sub"),
                helper("sup"),
                helper("span"),
                helper("blockquote"),
                helper("abbr"),
                helper("poem"),
                helper("syntaxhighlight")
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
                helper("score"),
                helper("code")
            )),
            |_| String::new()
        )
    ))(input)
}

// Parse divs and get rid of them
fn no_content_tag_parser(input: &str) -> IResult<&str, String> {
    let helper = |tag_name| {
        alt((
            map(
                tuple((
                    tag("<"),
                    tag(tag_name),
                    take_until(">"),
                    tag(">")
                )),
                |_| String::new()
            ),
            map(
                tuple((
                    tag("</"),
                    tag(tag_name),
                    take_until(">"),
                    tag(">")
                )),
                |_| String::new()
            )
        ))
    };

    // Allow for an early bailout
    let (input, _) = peek(tag("<"))(input)?;
    alt((
        helper("div"),
        helper("br"),
        helper("onlyinclude")
    ))(input)
}

// Parser for when we are in a 2 quote string
fn two_quote_state(input: &str) -> IResult<&str, String> {
    map(
        tuple((
            many_till(
                anychar, 
                alt((
                    peek(tag("''")),
                    peek(tag("\n"))
                ))
            ),
            alt((
                preceded(
                    tag("'''"),
                    five_quote_state
                ),
                map(tag("''"), |_| String::new())
            ))
        )),
        |((chars, _), suffix)| chars.into_iter().collect::<String>() + &suffix
    )(input)
}

// Parser for when we are in a 3 quote string
fn three_quote_state(input: &str) -> IResult<&str, String> {
    map(
        tuple((
            many_till(
                anychar, 
                alt((
                    peek(tag("''")),
                    peek(tag("\n"))
                ))
            ),
            alt((
                map(tag("'''"), |_| String::new()),
                preceded(
                    tag("''"),
                    five_quote_state
                )
            ))
        )),
        |((chars, _), suffix)| chars.into_iter().collect::<String>() + &suffix
    )(input)
}

// Parser for when we are in a 5 quote string
fn five_quote_state(input: &str) -> IResult<&str, String> {
    map(
        tuple((
            many_till(
                anychar, 
                alt((
                    peek(tag("''")),
                    peek(tag("\n"))
                ))
            ),
            alt((
                map(tag("'''''"), |_| String::new()),
                preceded(
                    tag("'''"),
                    two_quote_state
                ),
                preceded(
                    tag("''"),
                    three_quote_state
                )
            ))
        )),
        |((chars, _), suffix)| chars.into_iter().collect::<String>() + &suffix
    )(input)
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