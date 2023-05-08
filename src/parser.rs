use nom::{branch::alt, combinator::map, multi::many0, IResult};

use crate::node::*;

pub fn parse<'a>(input: &'a str) -> IResult<&'a str, Vec<Node<'a>>> {
    many0(alt((
        map(parse_block, Node::Block),
        map(parse_inline, Node::Inline),
    )))(input)
}

fn parse_block<'a>(input: &'a str) -> IResult<&'a str, Block<'a>> {
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

fn parse_search<'a>(input: &'a str) -> IResult<&'a str, Search<'a>> {
    todo!()
}

fn parse_code_block<'a>(input: &'a str) -> IResult<&'a str, CodeBlock<'a>> {
    todo!()
}

fn parse_math_block<'a>(input: &'a str) -> IResult<&'a str, MathBlock<'a>> {
    todo!()
}

fn parse_inline<'a>(input: &'a str) -> IResult<&'a str, Inline<'a>> {
    todo!()
}

fn parse_unicode_emoji<'a>(input: &'a str) -> IResult<&'a str, UnicodeEmoji<'a>> {
    todo!()
}

fn parse_emoji_code<'a>(input: &'a str) -> IResult<&'a str, EmojiCode<'a>> {
    todo!()
}

fn parse_bold<'a>(input: &'a str) -> IResult<&'a str, Bold<'a>> {
    todo!()
}

fn parse_small<'a>(input: &'a str) -> IResult<&'a str, Small<'a>> {
    todo!()
}

fn parse_italic<'a>(input: &'a str) -> IResult<&'a str, Italic<'a>> {
    todo!()
}

fn parse_strike<'a>(input: &'a str) -> IResult<&'a str, Strike<'a>> {
    todo!()
}

fn parse_inline_code<'a>(input: &'a str) -> IResult<&'a str, InlineCode<'a>> {
    todo!()
}

fn parse_math_inline<'a>(input: &'a str) -> IResult<&'a str, MathInline<'a>> {
    todo!()
}

fn parse_mention<'a>(input: &'a str) -> IResult<&'a str, Mention<'a>> {
    todo!()
}

fn parse_hashtag<'a>(input: &'a str) -> IResult<&'a str, Hashtag<'a>> {
    todo!()
}

fn parse_url<'a>(input: &'a str) -> IResult<&'a str, Url<'a>> {
    todo!()
}

fn parse_link<'a>(input: &'a str) -> IResult<&'a str, Link<'a>> {
    todo!()
}

fn parse_fn<'a>(input: &'a str) -> IResult<&'a str, Fn<'a>> {
    todo!()
}

fn parse_plain<'a>(input: &'a str) -> IResult<&'a str, Plain<'a>> {
    todo!()
}

fn parse_text<'a>(input: &'a str) -> IResult<&'a str, Text<'a>> {
    todo!()
}
