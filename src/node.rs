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
pub struct Bold(Vec<Inline>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Small(Vec<Inline>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Italic(Vec<Inline>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Strike(Vec<Inline>);

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
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Fn {
    pub name: String,
    pub args: Vec<(String, Option<String>)>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Plain(pub Vec<Text>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Text {
    pub text: String,
}
