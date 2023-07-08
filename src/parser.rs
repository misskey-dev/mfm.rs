use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take, take_till1, take_until1},
    character::complete::{anychar, char, line_ending, not_line_ending, satisfy},
    combinator::{consumed, map, map_res, not, opt, peek, recognize, rest, success, value, verify},
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

/// Verifies if the previous character is line ending or empty.
fn line_begin<'a, E>(last_char: Option<char>) -> impl FnMut(&'a str) -> IResult<&'a str, (), E>
where
    E: ParseError<&'a str>,
{
    verify(success(()), move |_| {
        if let Some(c) = last_char {
            "\r\n".contains(c)
        } else {
            true
        }
    })
}

/// Verifies if the next character is line ending or empty.
fn line_end<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    alt((
        value((), verify(rest, |s: &str| s.is_empty())),
        value((), peek(line_ending)),
    ))(input)
}

/// [`many0`] but remembers last character consumed in the last iteration.
///
/// The child parser receives last character as the second argument.
fn many0_keep_last_char<'a, O, E, F>(mut f: F) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<O>, E>
where
    F: FnMut(&'a str, Option<char>) -> IResult<&'a str, O, E>,
    E: ParseError<&'a str>,
{
    move |mut i| {
        let mut acc = Vec::with_capacity(4);
        let mut last_char = None;
        loop {
            let len = i.len();
            let res = consumed(|s| f(s, last_char))(i);
            match res {
                Err(nom::Err::Error(_)) => return Ok((i, acc)),
                Err(e) => return Err(e),
                Ok((i1, (consumed_input, o))) => {
                    // infinite loop check: the parser must always consume
                    if i1.len() == len {
                        return Err(nom::Err::Error(E::from_error_kind(i, ErrorKind::Many0)));
                    }

                    i = i1;
                    acc.push(o);
                    last_char = consumed_input.chars().last();
                }
            }
        }
    }
}

fn many1_keep_last_char<'a, O, E, F>(f: F) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<O>, E>
where
    F: FnMut(&'a str, Option<char>) -> IResult<&'a str, O, E>,
    E: ParseError<&'a str>,
{
    verify(many0_keep_last_char(f), |v: &Vec<O>| !v.is_empty())
}

/// Parser for partial MFM syntax.
#[derive(Clone, Debug)]
pub struct SimpleParser {}

impl SimpleParser {
    /// Returns a simple MFM node tree.
    pub fn parse<'a>(input: &'a str) -> IResult<&'a str, Vec<Simple>> {
        map(
            many0(alt((
                map(FullParser::parse_emoji_code, Simple::EmojiCode),
                map(FullParser::parse_text, Simple::Text),
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
            many0_keep_last_char(|i, last_char| {
                alt((
                    map(|s| self.parse_block(s, last_char), Node::Block),
                    map(|s| self.parse_inline(s, last_char), Node::Inline),
                ))(i)
            }),
            merge_text,
        )(input)
    }

    fn parse_block<'a>(&self, input: &'a str, last_char: Option<char>) -> IResult<&'a str, Block> {
        alt((
            map(|s| self.parse_quote(s, last_char), Block::Quote),
            map(|s| Self::parse_search(s, last_char), Block::Search),
            map(|s| Self::parse_code_block(s, last_char), Block::CodeBlock),
            map(|s| Self::parse_math_block(s, last_char), Block::MathBlock),
            map(|s| self.parse_center(s, last_char), Block::Center),
        ))(input)
    }

    fn parse_quote<'a>(&self, input: &'a str, last_char: Option<char>) -> IResult<&'a str, Quote> {
        delimited(
            alt((
                value((), many_m_n(1, 2, line_ending)),
                line_begin(last_char),
            )),
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

    fn parse_search<'a>(input: &'a str, last_char: Option<char>) -> IResult<&'a str, Search> {
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
            alt((value((), line_ending), line_begin(last_char))),
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

    fn parse_code_block<'a>(
        input: &'a str,
        last_char: Option<char>,
    ) -> IResult<&'a str, CodeBlock> {
        const MARK: &str = "```";
        delimited(
            alt((value((), line_ending), line_begin(last_char))),
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

    fn parse_math_block<'a>(
        input: &'a str,
        last_char: Option<char>,
    ) -> IResult<&'a str, MathBlock> {
        const OPEN: &str = r"\[";
        const CLOSE: &str = r"\]";
        delimited(
            alt((value((), line_ending), line_begin(last_char))),
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

    fn parse_center<'a>(
        &self,
        input: &'a str,
        last_char: Option<char>,
    ) -> IResult<&'a str, Center> {
        const OPEN: &str = "<center>";
        const CLOSE: &str = "</center>";
        map_res(
            delimited(
                alt((value((), line_ending), line_begin(last_char))),
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
                    map(
                        many1_keep_last_char(|i, last_char| inner.parse_inline(i, last_char)),
                        merge_text_inline,
                    )(contents)
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

    fn parse_inline<'a>(
        &self,
        input: &'a str,
        last_char: Option<char>,
    ) -> IResult<&'a str, Inline> {
        alt((
            map(Self::parse_emoji_code, Inline::EmojiCode),
            map(|s| self.parse_big(s), Inline::Fn),
            map(|s| self.parse_bold(s), Inline::Bold),
            map(|s| self.parse_small(s), Inline::Small),
            map(|s| self.parse_italic(s, last_char), Inline::Italic),
            map(|s| self.parse_strike(s), Inline::Strike),
            map(Self::parse_inline_code, Inline::InlineCode),
            map(Self::parse_math_inline, Inline::MathInline),
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

    // deprecated?
    fn parse_big<'a>(&self, input: &'a str) -> IResult<&'a str, Fn> {
        const MARK: &str = "***";
        map(
            delimited(
                tag(MARK),
                |contents| {
                    if let Some(inner) = self.nest() {
                        map(
                            many1_keep_last_char(|i, last_char| {
                                preceded(not(tag(MARK)), |s| inner.parse_inline(s, last_char))(i)
                            }),
                            merge_text_inline,
                        )(contents)
                    } else {
                        map(take_until1(MARK), |s: &str| {
                            vec![Inline::Text(Text {
                                text: s.to_string(),
                            })]
                        })(contents)
                    }
                },
                tag(MARK),
            ),
            |children| Fn {
                name: "tada".to_string(),
                args: Vec::new(),
                children,
            },
        )(input)
    }

    fn parse_bold<'a>(&self, input: &'a str) -> IResult<&'a str, Bold> {
        let bold_asta = |input: &'a str| -> IResult<&'a str, Vec<Inline>> {
            const MARK: &str = "**";
            delimited(
                tag(MARK),
                |contents| {
                    if let Some(inner) = self.nest() {
                        map(
                            many1_keep_last_char(|i, last_char| {
                                (preceded(not(tag(MARK)), |s| inner.parse_inline(s, last_char)))(i)
                            }),
                            merge_text_inline,
                        )(contents)
                    } else {
                        map(take_until1(MARK), |s: &str| {
                            vec![Inline::Text(Text {
                                text: s.to_string(),
                            })]
                        })(contents)
                    }
                },
                tag(MARK),
            )(input)
        };
        let bold_tag = |input: &'a str| -> IResult<&'a str, Vec<Inline>> {
            const OPEN: &str = "<b>";
            const CLOSE: &str = "</b>";
            delimited(
                tag(OPEN),
                |contents| {
                    if let Some(inner) = self.nest() {
                        map(
                            many1_keep_last_char(|i, last_char| {
                                preceded(not(tag(CLOSE)), |s| inner.parse_inline(s, last_char))(i)
                            }),
                            merge_text_inline,
                        )(contents)
                    } else {
                        map(take_until1(CLOSE), |s: &str| {
                            vec![Inline::Text(Text {
                                text: s.to_string(),
                            })]
                        })(contents)
                    }
                },
                tag(CLOSE),
            )(input)
        };
        let bold_under = |input: &'a str| -> IResult<&'a str, Vec<Inline>> {
            const MARK: &str = "__";
            delimited(
                tag(MARK),
                map(
                    take_till1(|c: char| !c.is_ascii_alphanumeric() && !" \t".contains(c)),
                    |s: &str| {
                        vec![Inline::Text(Text {
                            text: s.to_string(),
                        })]
                    },
                ),
                tag(MARK),
            )(input)
        };

        map(alt((bold_asta, bold_tag, bold_under)), Bold)(input)
    }

    fn parse_small<'a>(&self, input: &'a str) -> IResult<&'a str, Small> {
        const OPEN: &str = "<small>";
        const CLOSE: &str = "</small>";
        map(
            delimited(
                tag(OPEN),
                |contents| {
                    if let Some(inner) = self.nest() {
                        map(
                            many1_keep_last_char(|i, last_char| {
                                preceded(not(tag(CLOSE)), |s| inner.parse_inline(s, last_char))(i)
                            }),
                            merge_text_inline,
                        )(contents)
                    } else {
                        map(take_until1(CLOSE), |s: &str| {
                            vec![Inline::Text(Text {
                                text: s.to_string(),
                            })]
                        })(contents)
                    }
                },
                tag(CLOSE),
            ),
            Small,
        )(input)
    }

    fn parse_italic<'a>(
        &self,
        input: &'a str,
        last_char: Option<char>,
    ) -> IResult<&'a str, Italic> {
        let italic_tag = |input: &'a str| -> IResult<&'a str, Vec<Inline>> {
            const OPEN: &str = "<i>";
            const CLOSE: &str = "</i>";
            delimited(
                tag(OPEN),
                |contents| {
                    if let Some(inner) = self.nest() {
                        map(
                            many1_keep_last_char(|i, last_char| {
                                preceded(not(tag(CLOSE)), |s| inner.parse_inline(s, last_char))(i)
                            }),
                            merge_text_inline,
                        )(contents)
                    } else {
                        map(take_until1(CLOSE), |s: &str| {
                            vec![Inline::Text(Text {
                                text: s.to_string(),
                            })]
                        })(contents)
                    }
                },
                tag(CLOSE),
            )(input)
        };
        let italic_asta = |input: &'a str| -> IResult<&'a str, Vec<Inline>> {
            const MARK: &str = "*";
            delimited(
                tag(MARK),
                map(
                    take_till1(|c: char| !c.is_ascii_alphanumeric() && !" \t".contains(c)),
                    |s: &str| {
                        vec![Inline::Text(Text {
                            text: s.to_string(),
                        })]
                    },
                ),
                tag(MARK),
            )(input)
        };
        let italic_under = |input: &'a str| -> IResult<&'a str, Vec<Inline>> {
            const MARK: &str = "_";
            delimited(
                tag(MARK),
                map(
                    take_till1(|c: char| !c.is_ascii_alphanumeric() && !" \t".contains(c)),
                    |s: &str| {
                        vec![Inline::Text(Text {
                            text: s.to_string(),
                        })]
                    },
                ),
                tag(MARK),
            )(input)
        };

        map(
            alt((
                italic_tag,
                preceded(
                    verify(success(()), |_| {
                        if let Some(c) = last_char {
                            // check if character before the mark is not alnum
                            if c.is_ascii_alphanumeric() {
                                return false;
                            }
                        }
                        true
                    }),
                    alt((italic_asta, italic_under)),
                ),
            )),
            Italic,
        )(input)
    }

    fn parse_strike<'a>(&self, input: &'a str) -> IResult<&'a str, Strike> {
        let strike_tag = |input: &'a str| -> IResult<&'a str, Vec<Inline>> {
            const OPEN: &str = "<s>";
            const CLOSE: &str = "</s>";
            delimited(
                tag(OPEN),
                |contents| {
                    if let Some(inner) = self.nest() {
                        map(
                            many1(preceded(not(tag(CLOSE)), |s| inner.parse_inline(s, None))),
                            merge_text_inline,
                        )(contents)
                    } else {
                        map(take_until1(CLOSE), |s: &str| {
                            vec![Inline::Text(Text {
                                text: s.to_string(),
                            })]
                        })(contents)
                    }
                },
                tag(CLOSE),
            )(input)
        };
        let strike_wave = |input: &'a str| -> IResult<&'a str, Vec<Inline>> {
            const MARK: &str = "~~";
            delimited(
                tag(MARK),
                |contents| {
                    if let Some(inner) = self.nest() {
                        map(
                            many1(preceded(not(alt((tag(MARK), line_ending))), |s| {
                                inner.parse_inline(s, None)
                            })),
                            merge_text_inline,
                        )(contents)
                    } else {
                        map(
                            recognize(many1(preceded(not(alt((tag(MARK), line_ending))), anychar))),
                            |text: &str| {
                                vec![Inline::Text(Text {
                                    text: text.to_string(),
                                })]
                            },
                        )(contents)
                    }
                },
                tag(MARK),
            )(input)
        };

        map(alt((strike_tag, strike_wave)), Strike)(input)
    }

    fn parse_inline_code<'a>(input: &'a str) -> IResult<&'a str, InlineCode> {
        delimited(
            char('`'),
            map(
                recognize(many1(preceded(
                    not(alt((
                        value((), char('`')),
                        value((), char('´')),
                        value((), line_ending),
                    ))),
                    anychar,
                ))),
                |code: &str| InlineCode {
                    code: code.to_string(),
                },
            ),
            char('`'),
        )(input)
    }

    fn parse_math_inline<'a>(input: &'a str) -> IResult<&'a str, MathInline> {
        const OPEN: &str = r"\(";
        const CLOSE: &str = r"\)";
        delimited(
            tag(OPEN),
            map(
                recognize(many1(preceded(
                    not(alt((tag(CLOSE), line_ending))),
                    anychar,
                ))),
                |formula: &str| MathInline {
                    formula: formula.to_string(),
                },
            ),
            tag(CLOSE),
        )(input)
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
