use crate::node::{Inline, Node, Simple, Text};

/// Pushes text to vector if stored string is not empty.
fn generate_text<T: From<Text>>(mut dest: Vec<T>, stored_string: String) -> (Vec<T>, String) {
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

pub(crate) trait MergeText {
    fn merge_text(nodes: Vec<Self>) -> Vec<Self>
    where
        Self: Sized;
}

impl MergeText for Node {
    /// Merges adjacent text nodes into one with their contents concatenated.
    fn merge_text(nodes: Vec<Self>) -> Vec<Self> {
        let (dest, stored_string) = nodes.into_iter().fold(
            (Vec::<Self>::new(), String::new()),
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
}

impl MergeText for Inline {
    /// Merges adjacent text nodes into one with their contents concatenated.
    fn merge_text(nodes: Vec<Self>) -> Vec<Self> {
        let (dest, stored_string) = nodes.into_iter().fold(
            (Vec::<Self>::new(), String::new()),
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
}

impl MergeText for Simple {
    /// Merges adjacent text nodes into one with their contents concatenated.
    fn merge_text(nodes: Vec<Self>) -> Vec<Self> {
        let (dest, stored_string) = nodes.into_iter().fold(
            (Vec::<Self>::new(), String::new()),
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
}
