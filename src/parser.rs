use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take, take_till1},
    character::complete::{anychar, char, line_ending, not_line_ending, satisfy},
    combinator::{map, map_res, not, opt, peek, recognize, rest, value, verify},
    error::{ErrorKind, ParseError},
    multi::{many0, many1, many_m_n, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    IResult,
};

use crate::{
    node::*,
    util::{merge_text, merge_text_inline, merge_text_simple},
};

fn failure<'a>(input: &'a str) -> nom::error::Error<&'a str> {
    nom::error::Error::from_error_kind(input, ErrorKind::Fail)
}

fn space<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    alt((tag("\u{0020}"), tag("\u{3000}"), tag("\t")))(input)
}

/// Verifies if the next character is line ending or empty.
fn line_end<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    alt((
        value((), verify(rest, |s: &str| s.is_empty())),
        value((), peek(line_ending)),
    ))(input)
}

/// Parser for partial MFM syntax.
#[derive(Clone, Debug)]
pub struct SimpleParser {}

impl SimpleParser {
    /// Returns a simple MFM node tree.
    pub fn parse<'a>(input: &'a str) -> IResult<&'a str, Vec<Simple>> {
        map(
            many0(alt((
                map(|s| FullParser::parse_emoji_code(s), Simple::EmojiCode),
                map(|s| FullParser::parse_text(s), Simple::Text),
            ))),
            merge_text_simple,
        )(input)
    }
}

/// Parser for full MFM syntax.
#[derive(Clone, Debug)]
pub struct FullParser {
    nest_limit: u32,
    depth: u32,
}

impl Default for FullParser {
    fn default() -> Self {
        FullParser {
            nest_limit: 20,
            depth: 0,
        }
    }
}

impl FullParser {
    /// Creates a parser with nest limit.
    pub fn new(nest_limit: u32) -> Self {
        FullParser {
            nest_limit,
            depth: 0,
        }
    }

    /// Returns a parser if its depth does not reach the nest limit.
    fn nest(&self) -> Option<Self> {
        let depth = self.depth + 1;
        if depth < self.nest_limit {
            Some(FullParser {
                nest_limit: self.nest_limit,
                depth,
            })
        } else {
            None
        }
    }

    /// Returns a full MFM node tree.
    pub fn parse<'a>(&self, input: &'a str) -> IResult<&'a str, Vec<Node>> {
        map(
            many0(alt((
                map(|s| self.parse_block(s), Node::Block),
                map(|s| self.parse_inline(s), Node::Inline),
            ))),
            merge_text,
        )(input)
    }

    fn parse_block<'a>(&self, input: &'a str) -> IResult<&'a str, Block> {
        alt((
            map(|s| self.parse_quote(s), Block::Quote),
            map(Self::parse_search, Block::Search),
            map(Self::parse_code_block, Block::CodeBlock),
            map(Self::parse_math_block, Block::MathBlock),
            map(|s| self.parse_center(s), Block::Center),
        ))(input)
    }

    fn parse_quote<'a>(&self, input: &'a str) -> IResult<&'a str, Quote> {
        delimited(
            many_m_n(0, 2, line_ending),
            map_res(
                map(
                    verify(
                        separated_list1(
                            line_ending,
                            preceded(pair(char('>'), many0(space)), not_line_ending),
                        ),
                        // disallow empty content if single line
                        |contents: &Vec<&str>| contents.len() > 1 || contents[0].len() != 0,
                    ),
                    |contents| contents.join("\n"),
                ),
                |contents| {
                    // parse inner contents
                    let nodes = if let Some(inner) = self.nest() {
                        inner.parse(&contents).map_err(|_| failure(input))?.1
                    } else {
                        vec![Node::Inline(Inline::Text(Text { text: contents }))]
                    };
                    Ok::<Quote, nom::error::Error<&str>>(Quote(nodes))
                },
            ),
            many_m_n(0, 2, line_ending),
        )(input)
    }

    fn parse_search<'a>(input: &'a str) -> IResult<&'a str, Search> {
        fn button<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
            recognize(alt((
                delimited(
                    char('['),
                    alt((tag("検索"), tag_no_case("search"))),
                    char(']'),
                ),
                alt((tag("検索"), tag_no_case("search"))),
            )))(input)
        }

        delimited(
            opt(line_ending),
            map(
                tuple((
                    recognize(many1(preceded(
                        not(alt((
                            value((), line_ending),
                            value((), tuple((space, button, line_end))),
                        ))),
                        anychar,
                    ))),
                    space,
                    button,
                )),
                |(query, space, button)| {
                    let content = format!("{query}{space}{button}");
                    Search {
                        query: query.to_string(),
                        content,
                    }
                },
            ),
            opt(line_ending),
        )(input)
    }

    fn parse_code_block<'a>(input: &'a str) -> IResult<&'a str, CodeBlock> {
        const MARK: &str = "```";
        delimited(
            opt(line_ending),
            delimited(
                tag(MARK),
                map(
                    separated_pair(
                        opt(verify(not_line_ending, |s: &str| s.len() > 0)),
                        line_ending,
                        recognize(many1(preceded(
                            not(tuple((line_ending, tag(MARK), line_end))),
                            anychar,
                        ))),
                    ),
                    |(lang, code)| CodeBlock {
                        code: code.to_string(),
                        lang: lang.map(String::from),
                    },
                ),
                tuple((line_ending, tag(MARK), line_end)),
            ),
            opt(line_ending),
        )(input)
    }

    fn parse_math_block<'a>(input: &'a str) -> IResult<&'a str, MathBlock> {
        const OPEN: &str = r"\[";
        const CLOSE: &str = r"\]";
        delimited(
            opt(line_ending),
            delimited(
                tag(OPEN),
                delimited(
                    opt(line_ending),
                    map(
                        recognize(many1(preceded(
                            not(pair(opt(line_ending), tag(CLOSE))),
                            anychar,
                        ))),
                        |formula: &str| MathBlock {
                            formula: formula.to_string(),
                        },
                    ),
                    opt(line_ending),
                ),
                pair(tag(CLOSE), line_end),
            ),
            opt(line_ending),
        )(input)
    }

    fn parse_center<'a>(&self, input: &'a str) -> IResult<&'a str, Center> {
        const OPEN: &str = "<center>";
        const CLOSE: &str = "</center>";
        map_res(
            delimited(
                opt(line_ending),
                delimited(
                    tag(OPEN),
                    delimited(
                        opt(line_ending),
                        recognize(many1(preceded(
                            not(pair(opt(line_ending), tag(CLOSE))),
                            anychar,
                        ))),
                        opt(line_ending),
                    ),
                    pair(tag(CLOSE), line_end),
                ),
                opt(line_ending),
            ),
            |contents| {
                let nodes = if let Some(inner) = self.nest() {
                    map(many1(|s| inner.parse_inline(s)), merge_text_inline)(contents)
                        .map_err(|_| failure(contents))?
                        .1
                } else {
                    vec![Inline::Text(Text {
                        text: contents.to_string(),
                    })]
                };
                Ok::<Center, nom::error::Error<&str>>(Center(nodes))
            },
        )(input)
    }

    fn parse_inline<'a>(&self, input: &'a str) -> IResult<&'a str, Inline> {
        alt((
            map(Self::parse_emoji_code, Inline::EmojiCode),
            map(Self::parse_plain, Inline::Plain),
            map(Self::parse_text, Inline::Text),
        ))(input)
    }

    fn parse_unicode_emoji<'a>(input: &'a str) -> IResult<&'a str, UnicodeEmoji> {
        todo!()
    }

    fn parse_emoji_code<'a>(input: &'a str) -> IResult<&'a str, EmojiCode> {
        fn side<'a>(input: &'a str) -> IResult<&'a str, ()> {
            not(satisfy(|c| c.is_ascii_alphanumeric()))(input)
        }

        delimited(
            side,
            delimited(
                char(':'),
                map(
                    take_till1(|c: char| !c.is_ascii_alphanumeric() && !"_+-".contains(c)),
                    |s: &str| EmojiCode {
                        name: s.to_string(),
                    },
                ),
                char(':'),
            ),
            side,
        )(input)
    }

    fn parse_bold<'a>(input: &'a str) -> IResult<&'a str, Bold> {
        todo!()
    }

    fn parse_small<'a>(input: &'a str) -> IResult<&'a str, Small> {
        todo!()
    }

    fn parse_italic<'a>(input: &'a str) -> IResult<&'a str, Italic> {
        todo!()
    }

    fn parse_strike<'a>(input: &'a str) -> IResult<&'a str, Strike> {
        todo!()
    }

    fn parse_inline_code<'a>(input: &'a str) -> IResult<&'a str, InlineCode> {
        todo!()
    }

    fn parse_math_inline<'a>(input: &'a str) -> IResult<&'a str, MathInline> {
        todo!()
    }

    fn parse_mention<'a>(input: &'a str) -> IResult<&'a str, Mention> {
        todo!()
    }

    fn parse_hashtag<'a>(input: &'a str) -> IResult<&'a str, Hashtag> {
        todo!()
    }

    fn parse_url<'a>(input: &'a str) -> IResult<&'a str, Url> {
        todo!()
    }

    fn parse_link<'a>(input: &'a str) -> IResult<&'a str, Link> {
        todo!()
    }

    fn parse_fn<'a>(input: &'a str) -> IResult<&'a str, Fn> {
        todo!()
    }

    fn parse_plain<'a>(input: &'a str) -> IResult<&'a str, Plain> {
        const OPEN: &str = "<plain>";
        const CLOSE: &str = "</plain>";
        delimited(
            tag(OPEN),
            delimited(
                opt(line_ending),
                map(
                    recognize(many1(preceded(
                        not(pair(opt(line_ending), tag(CLOSE))),
                        anychar,
                    ))),
                    |text: &str| {
                        Plain(vec![Text {
                            text: text.to_string(),
                        }])
                    },
                ),
                opt(line_ending),
            ),
            tag(CLOSE),
        )(input)
    }

    fn parse_text<'a>(input: &'a str) -> IResult<&'a str, Text> {
        map(take(1u8), |s: &str| Text {
            text: s.to_string(),
        })(input)
    }
}
