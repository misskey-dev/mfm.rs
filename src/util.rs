use crate::node::{Inline, Node, Simple, Text};

/// Pushes text to vector if stored string is not empty.
fn generate_text<T>(mut dest: Vec<T>, stored_string: String) -> (Vec<T>, String)
where
    T: From<Text> + TryInto<Text>,
{
    if !stored_string.is_empty() {
        let text = Text {
            text: stored_string.clone(),
        };
        dest.push(text.into());
        (dest, String::new())
    } else {
        (dest, stored_string)
    }
}

/// Merges adjacent text nodes into one with their contents concatenated.
pub fn merge_text(nodes: Vec<Node>) -> Vec<Node> {
    let (dest, stored_string) = nodes.into_iter().fold(
        (Vec::<Node>::new(), String::new()),
        |(dest, stored_string), node| {
            if let Node::Inline(Inline::Text(Text { text })) = node {
                (dest, stored_string + &text)
            } else {
                let (mut dest, stored_string) = generate_text(dest, stored_string);
                dest.push(node);
                (dest, stored_string)
            }
        },
    );

    generate_text(dest, stored_string).0
}

/// Merges adjacent inline text nodes into one with their contents concatenated.
pub fn merge_text_inline(nodes: Vec<Inline>) -> Vec<Inline> {
    let (dest, stored_string) = nodes.into_iter().fold(
        (Vec::<Inline>::new(), String::new()),
        |(dest, stored_string), node| {
            if let Inline::Text(Text { text }) = node {
                (dest, stored_string + &text)
            } else {
                let (mut dest, stored_string) = generate_text(dest, stored_string);
                dest.push(node);
                (dest, stored_string)
            }
        },
    );

    generate_text(dest, stored_string).0
}

/// Merges adjacent simple text nodes into one with their contents concatenated.
pub fn merge_text_simple(nodes: Vec<Simple>) -> Vec<Simple> {
    let (dest, stored_string) = nodes.into_iter().fold(
        (Vec::<Simple>::new(), String::new()),
        |(dest, stored_string), node| {
            if let Simple::Text(Text { text }) = node {
                (dest, stored_string + &text)
            } else {
                let (mut dest, stored_string) = generate_text(dest, stored_string);
                dest.push(node);
                (dest, stored_string)
            }
        },
    );

    generate_text(dest, stored_string).0
}
