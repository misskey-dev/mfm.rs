#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Node {
    Block(Block),
    Inline(Inline),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Block {
    Quote(Quote),
    Search(Search),
    CodeBlock(CodeBlock),
    MathBlock(MathBlock),
    Center(Center),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Quote(pub Vec<Node>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Search {
    pub query: String,
    pub content: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CodeBlock {
    pub code: String,
    pub lang: Option<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MathBlock {
    pub formula: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Center(pub Vec<Inline>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Inline {
    UnicodeEmoji(UnicodeEmoji),
    EmojiCode(EmojiCode),
    Bold(Bold),
    Small(Small),
    Italic(Italic),
    Strike(Strike),
    InlineCode(InlineCode),
    MathInline(MathInline),
    Mention(Mention),
    Hashtag(Hashtag),
    Url(Url),
    Link(Link),
    Fn(Fn),
    Plain(Plain),
    Text(Text),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Simple {
    UnicodeEmoji(UnicodeEmoji),
    EmojiCode(EmojiCode),
    Text(Text),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UnicodeEmoji {
    pub emoji: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EmojiCode {
    pub name: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Bold(pub Vec<Inline>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Small(pub Vec<Inline>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Italic(pub Vec<Inline>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Strike(pub Vec<Inline>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InlineCode {
    pub code: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MathInline {
    pub formula: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Mention {
    pub username: String,
    pub host: Option<String>,
    pub acct: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Hashtag {
    pub hashtag: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Url {
    pub url: String,
    pub brackets: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Link {
    pub url: String,
    pub silent: bool,
    pub children: Vec<Inline>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Fn {
    pub name: String,
    pub args: Vec<(String, Option<String>)>,
    pub children: Vec<Inline>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Plain(pub Vec<Text>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Text {
    pub text: String,
}

impl From<Text> for Node {
    fn from(text: Text) -> Self {
        Node::Inline(Inline::Text(text))
    }
}

impl From<Text> for Inline {
    fn from(text: Text) -> Self {
        Inline::Text(text)
    }
}

impl From<Text> for Simple {
    fn from(text: Text) -> Self {
        Simple::Text(text)
    }
}

impl TryFrom<Node> for Text {
    type Error = ();

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        if let Node::Inline(Inline::Text(text)) = node {
            Ok(text)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Inline> for Text {
    type Error = ();

    fn try_from(inline: Inline) -> Result<Self, Self::Error> {
        if let Inline::Text(text) = inline {
            Ok(text)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Simple> for Text {
    type Error = ();

    fn try_from(inline: Simple) -> Result<Self, Self::Error> {
        if let Simple::Text(text) = inline {
            Ok(text)
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ValueOrText<T> {
    Value(T),
    Text(Text),
}
