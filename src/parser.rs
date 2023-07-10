use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case, take, take_till, take_till1, take_until1},
    character::complete::{anychar, char, line_ending, not_line_ending, satisfy},
    combinator::{
        consumed, fail, map, map_res, not, opt, peek, recognize, rest, success, value, verify,
    },
    error::{ErrorKind, ParseError},
    multi::{many0, many1, many_m_n, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use nom_regex::{lib::regex::Regex, str::re_find};
use once_cell::sync::Lazy;
use unicode_segmentation::UnicodeSegmentation;

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

/// Verifies if the successing character is not on the same line as the match.
fn line_end<'a, O, E, F>(parser: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    E: ParseError<&'a str>,
    F: nom::Parser<&'a str, O, E>,
{
    terminated(
        parser,
        alt((
            value((), verify(rest, |s: &str| s.is_empty())),
            value((), peek(line_ending)),
        )),
    )
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
                map(FullParser::parse_unicode_emoji, Simple::UnicodeEmoji),
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
    link_label: bool,
}

impl Default for FullParser {
    fn default() -> Self {
        FullParser {
            nest_limit: 20,
            depth: 0,
            link_label: false,
        }
    }
}

impl FullParser {
    /// Creates a parser with nest limit.
    pub fn new(nest_limit: u32) -> Self {
        FullParser {
            nest_limit,
            depth: 0,
            link_label: false,
        }
    }

    /// Returns a parser if its depth does not reach the nest limit.
    fn nest(&self) -> Option<Self> {
        let depth = self.depth + 1;
        if depth < self.nest_limit {
            Some(FullParser {
                nest_limit: self.nest_limit,
                depth,
                link_label: self.link_label,
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
                            value((), line_end(pair(space, button))),
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
                            not(line_end(pair(line_ending, tag(MARK)))),
                            anychar,
                        ))),
                    ),
                    |(lang, code)| CodeBlock {
                        code: code.to_string(),
                        lang: lang.map(String::from),
                    },
                ),
                line_end(pair(line_ending, tag(MARK))),
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
                line_end(tag(CLOSE)),
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
                    line_end(tag(CLOSE)),
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
            map(Self::parse_unicode_emoji, Inline::UnicodeEmoji),
            map(Self::parse_emoji_code, Inline::EmojiCode),
            map(|s| self.parse_big(s), Inline::Fn),
            map(|s| self.parse_bold(s), Inline::Bold),
            map(|s| self.parse_small(s), Inline::Small),
            map(|s| self.parse_italic(s, last_char), Inline::Italic),
            map(|s| self.parse_strike(s), Inline::Strike),
            map(Self::parse_inline_code, Inline::InlineCode),
            map(Self::parse_math_inline, Inline::MathInline),
            map(
                |s| self.parse_mention(s, last_char),
                |e| match e {
                    ValueOrText::Value(mention) => Inline::Mention(mention),
                    ValueOrText::Text(text) => Inline::Text(text),
                },
            ),
            map(|s| self.parse_hashtag(s, last_char), Inline::Hashtag),
            map(|s| self.parse_url(s), Inline::Url),
            map(|s| self.parse_link(s), Inline::Link),
            map(|s| self.parse_fn(s), Inline::Fn),
            map(Self::parse_plain, Inline::Plain),
            map(Self::parse_text, Inline::Text),
        ))(input)
    }

    fn parse_unicode_emoji<'a>(input: &'a str) -> IResult<&'a str, UnicodeEmoji> {
        if let Some(s) = input.graphemes(true).next() {
            if let Some(_) = emojis::get(s) {
                return Ok((
                    &input[s.len()..],
                    UnicodeEmoji {
                        emoji: s.to_string(),
                    },
                ));
            }
        }
        fail(input)
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

    fn parse_mention<'a>(
        &self,
        input: &'a str,
        last_char: Option<char>,
    ) -> IResult<&'a str, ValueOrText<Mention>> {
        // not empty username without trailing dashes
        fn username<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
            static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[[:word:]-]+[[:word:]]").unwrap());
            preceded(
                char('@'),
                preceded(not(char('-')), re_find(Lazy::force(&RE).clone())),
            )(input)
        }

        // not empty hostname without trailing dashes or dots
        fn hostname<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
            static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[[:word:].-]+[[:word:]]").unwrap());
            preceded(
                char('@'),
                preceded(
                    not(alt((char('-'), char('.')))),
                    re_find(Lazy::force(&RE).clone()),
                ),
            )(input)
        }

        fn valid<'a>(input: &'a str) -> IResult<&'a str, Mention> {
            map(pair(username, opt(hostname)), |(first, second)| Mention {
                username: first.to_string(),
                host: second.map(String::from),
                acct: if let Some(second) = second {
                    format!("@{first}@{second}")
                } else {
                    format!("@{first}")
                },
            })(input)
        }

        fn possibly_invalid<'a>(input: &'a str) -> IResult<&'a str, Text> {
            map(
                recognize(tuple((
                    char('@'),
                    take_till1(|c: char| !c.is_ascii_alphanumeric() && !"_-".contains(c)),
                    char('@'),
                    take_till1(|c: char| !c.is_ascii_alphanumeric() && !"_.-".contains(c)),
                ))),
                |s: &str| Text {
                    text: s.to_string(),
                },
            )(input)
        }

        let res_mention = preceded(
            verify(success(()), |_| {
                !self.link_label && last_char.map_or(true, |c| !c.is_ascii_alphanumeric())
            }),
            map(valid, ValueOrText::Value),
        )(input);
        if let Ok((_, ValueOrText::Value(Mention { host: Some(_), .. }))) = res_mention {
            res_mention
        } else {
            let res_text = map(possibly_invalid, ValueOrText::Text)(input);
            if res_text.is_ok() {
                // return Text to ignore the latter part
                res_text
            } else {
                res_mention
            }
        }
    }

    fn parse_hashtag<'a>(
        &self,
        input: &'a str,
        last_char: Option<char>,
    ) -> IResult<&'a str, Hashtag> {
        fn is_not_hashtag_char(c: char) -> bool {
            ".,!?'\"#:/[]【】()「」（）<>\u{0020}\u{3000}\t\r\n".contains(c)
        }

        fn inner_item<'a>(input: &'a str, depth: u32, nest_limit: u32) -> IResult<&'a str, ()> {
            fn nest<'a>(input: &'a str, depth: u32, nest_limit: u32) -> IResult<&'a str, ()> {
                let depth = depth + 1;
                if depth < nest_limit {
                    value((), many0(|s| inner_item(s, depth, nest_limit)))(input)
                } else {
                    value((), take_till(is_not_hashtag_char))(input)
                }
            }

            alt((
                delimited(char('('), |s| nest(s, depth, nest_limit), char(')')),
                delimited(char('['), |s| nest(s, depth, nest_limit), char(']')),
                delimited(char('「'), |s| nest(s, depth, nest_limit), char('」')),
                delimited(char('（'), |s| nest(s, depth, nest_limit), char('）')),
                value((), take_till1(is_not_hashtag_char)),
            ))(input)
        }

        preceded(
            verify(success(()), |_| {
                !self.link_label && last_char.map_or(true, |c| !c.is_ascii_alphanumeric())
            }),
            preceded(
                char('#'),
                map(
                    verify(
                        recognize(many1(|s| inner_item(s, self.depth, self.nest_limit))),
                        |s: &str| !s.chars().all(|c| c.is_ascii_digit()),
                    ),
                    |s| Hashtag {
                        hashtag: s.to_string(),
                    },
                ),
            ),
        )(input)
    }

    fn parse_url<'a>(&self, input: &'a str) -> IResult<&'a str, Url> {
        fn find_url_str<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
            static RE: Lazy<Regex> =
                Lazy::new(|| Regex::new(r"^[[:word:].,/:%#@$&?!~=+-]+").unwrap());
            re_find(Lazy::force(&RE).clone())(input)
        }

        fn inner_item<'a>(input: &'a str, depth: u32, nest_limit: u32) -> IResult<&'a str, ()> {
            fn nest<'a>(input: &'a str, depth: u32, nest_limit: u32) -> IResult<&'a str, ()> {
                let depth = depth + 1;
                if depth < nest_limit {
                    value((), many0(|s| inner_item(s, depth, nest_limit)))(input)
                } else {
                    value((), find_url_str)(input)
                }
            }

            alt((
                delimited(char('('), |s| nest(s, depth, nest_limit), char(')')),
                delimited(char('['), |s| nest(s, depth, nest_limit), char(']')),
                value((), alt((tag("()"), tag("[]"), find_url_str))),
            ))(input)
        }

        let url = |input: &'a str| -> IResult<&'a str, Url> {
            let (_, trimmed) = map(
                pair(
                    recognize(tuple((tag("http"), opt(char('s')), tag("://")))),
                    verify(
                        map(
                            recognize(many1(|s| inner_item(s, self.depth, self.nest_limit))),
                            |s| s.trim_end_matches(&['.', ',']),
                        ),
                        |s: &str| !s.is_empty(),
                    ),
                ),
                |(first, second)| format!("{first}{second}"),
            )(input)?;
            Ok((
                // include trailing dots and commas to remaining value
                &input[trimmed.len()..],
                Url {
                    url: trimmed.to_string(),
                    brackets: false,
                },
            ))
        };

        fn url_alt<'a>(input: &'a str) -> IResult<&'a str, Url> {
            delimited(
                char('<'),
                map(
                    recognize(tuple((
                        tag("http"),
                        opt(char('s')),
                        tag("://"),
                        many1(preceded(
                            not(alt((value((), char('>')), value((), space)))),
                            anychar,
                        )),
                    ))),
                    |s| Url {
                        url: s.to_string(),
                        brackets: true,
                    },
                ),
                char('>'),
            )(input)
        }

        preceded(
            verify(success(()), |_| !self.link_label),
            alt((url, url_alt)),
        )(input)
    }

    fn parse_link<'a>(&self, input: &'a str) -> IResult<&'a str, Link> {
        preceded(
            verify(success(()), |_| !self.link_label),
            map(
                tuple((
                    opt(char('?')),
                    delimited(
                        char('['),
                        |contents| {
                            if let Some(FullParser {
                                nest_limit, depth, ..
                            }) = self.nest()
                            {
                                let inner = FullParser {
                                    nest_limit,
                                    depth,
                                    link_label: true,
                                };
                                let res = map(
                                    many1(preceded(
                                        not(alt((value((), char(']')), value((), line_ending)))),
                                        |s| inner.parse_inline(s, None),
                                    )),
                                    merge_text_inline,
                                )(contents);
                                res
                            } else {
                                map(is_not("]\r\n"), |s: &str| {
                                    vec![Inline::Text(Text {
                                        text: s.to_string(),
                                    })]
                                })(contents)
                            }
                        },
                        char(']'),
                    ),
                    delimited(char('('), |s| self.parse_url(s), char(')')),
                )),
                |(silent, label, url)| Link {
                    url: url.url,
                    silent: silent.is_some(),
                    children: label,
                },
            ),
        )(input)
    }

    fn parse_fn<'a>(&self, input: &'a str) -> IResult<&'a str, Fn> {
        delimited(
            tag("$["),
            map(
                separated_pair(
                    pair(
                        // name
                        take_till1(|c: char| !c.is_ascii_alphanumeric() && c != '_'),
                        // args
                        opt(preceded(
                            char('.'),
                            separated_list1(
                                char(','),
                                map(
                                    pair(
                                        // arg name
                                        take_till1(|c: char| {
                                            !c.is_ascii_alphanumeric() && c != '_'
                                        }),
                                        // arg value
                                        opt(preceded(
                                            char('='),
                                            take_till1(|c: char| {
                                                !c.is_ascii_alphanumeric() && !"_.-".contains(c)
                                            }),
                                        )),
                                    ),
                                    |(name, value): (&str, Option<&str>)| {
                                        (name.to_string(), value.map(String::from))
                                    },
                                ),
                            ),
                        )),
                    ),
                    char(' '),
                    |contents| {
                        if let Some(inner) = self.nest() {
                            map(
                                many1(preceded(not(char(']')), |s| inner.parse_inline(s, None))),
                                merge_text_inline,
                            )(contents)
                        } else {
                            map(is_not("]"), |s: &str| {
                                vec![Inline::Text(Text {
                                    text: s.to_string(),
                                })]
                            })(contents)
                        }
                    },
                ),
                |((name, args), children)| Fn {
                    name: name.to_string(),
                    args: args.unwrap_or_else(Vec::new),
                    children,
                },
            ),
            char(']'),
        )(input)
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
