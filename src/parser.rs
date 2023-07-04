use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{anychar, char, line_ending, not_line_ending},
    combinator::{map, map_res, not, opt, recognize, verify},
    error::{ErrorKind, ParseError},
    multi::{many0, many1, many_m_n, separated_list1},
    sequence::{delimited, pair, preceded},
    IResult,
};

use crate::{node::*, util::merge_text};

fn failure<'a>(input: &'a str) -> nom::error::Error<&'a str> {
    nom::error::Error::from_error_kind(input, ErrorKind::Fail)
}

fn space<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    alt((tag("\u{0020}"), tag("\u{3000}"), tag("\t")))(input)
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
            // map(Self::parse_search, Block::Search),
            // map(Self::parse_code_block, Block::CodeBlock),
            // map(Self::parse_math_block, Block::MathBlock),
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
        todo!()
    }

    fn parse_code_block<'a>(input: &'a str) -> IResult<&'a str, CodeBlock> {
        todo!()
    }

    fn parse_math_block<'a>(input: &'a str) -> IResult<&'a str, MathBlock> {
        todo!()
    }

    fn parse_inline<'a>(&self, input: &'a str) -> IResult<&'a str, Inline> {
        alt((
            map(Self::parse_plain, Inline::Plain),
            map(Self::parse_text, Inline::Text),
        ))(input)
    }

    fn parse_unicode_emoji<'a>(input: &'a str) -> IResult<&'a str, UnicodeEmoji> {
        todo!()
    }

    fn parse_emoji_code<'a>(input: &'a str) -> IResult<&'a str, EmojiCode> {
        todo!()
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
