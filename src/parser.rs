use nom::{branch::alt, combinator::map, multi::many0, IResult};

use crate::node::*;

pub fn parse<'a>(input: &'a str) -> IResult<&'a str, Vec<Node>> {
    many0(alt((
        map(parse_block, Node::Block),
        map(parse_inline, Node::Inline),
    )))(input)
}

fn parse_block<'a>(input: &'a str) -> IResult<&'a str, Block> {
    alt((
        map(parse_quote, Block::Quote),
        map(parse_search, Block::Search),
        map(parse_code_block, Block::CodeBlock),
        map(parse_math_block, Block::MathBlock),
    ))(input)
}

fn parse_quote<'a>(input: &'a str) -> IResult<&'a str, Quote> {
    todo!()
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

fn parse_inline<'a>(input: &'a str) -> IResult<&'a str, Inline> {
    todo!()
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
    todo!()
}

fn parse_text<'a>(input: &'a str) -> IResult<&'a str, Text> {
    todo!()
}
