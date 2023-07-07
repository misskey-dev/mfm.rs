use crate::{
    node::{Node, Simple},
    parser::{FullParser, SimpleParser},
};

/// Generates a MFM Node tree from the MFM string.
pub fn parse(input: &str) -> Result<Vec<Node>, nom::Err<nom::error::Error<&str>>> {
    FullParser::default().parse(input).map(|(_, nodes)| nodes)
}

/// Generates a MFM Node tree from the MFM string with a specific maximum nest depth.
pub fn parse_with_nest_limit(
    input: &str,
    nest_limit: u32,
) -> Result<Vec<Node>, nom::Err<nom::error::Error<&str>>> {
    FullParser::new(nest_limit)
        .parse(input)
        .map(|(_, nodes)| nodes)
}

/// Generates a MFM Simple Node tree from the MFM string.
pub fn parse_simple(input: &str) -> Result<Vec<Simple>, nom::Err<nom::error::Error<&str>>> {
    SimpleParser::parse(input).map(|(_, nodes)| nodes)
}
