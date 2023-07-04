use crate::node::{Inline, Node, Text};

/// Pushes text to vector if stored string is not empty.
fn generate_text(mut dest: Vec<Node>, stored_string: String) -> (Vec<Node>, String) {
    if !stored_string.is_empty() {
        let text = Node::Inline(Inline::Text(Text {
            text: stored_string.clone(),
        }));
        dest.push(text);
        (dest, String::new())
    } else {
        (dest, stored_string)
    }
}

/// Merges adjacent text nodes into one with their contents concatenated.
pub fn merge_text(nodes: Vec<Node>) -> Vec<Node> {
    let (dest, stored_string) = nodes.into_iter().fold(
        (Vec::<Node>::new(), String::new()),
        |(dest, stored_string), node| match node {
            Node::Inline(Inline::Text(Text { text })) => (dest, stored_string + &text),
            _ => {
                let (mut dest, stored_string) = generate_text(dest, stored_string);
                dest.push(node);
                (dest, stored_string)
            }
        },
    );

    generate_text(dest, stored_string).0
}
