#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Node<'a> {
    Block(Block<'a>),
    Inline(Inline<'a>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Block<'a> {
    Quote(Quote<'a>),
    Search(Search<'a>),
    CodeBlock(CodeBlock<'a>),
    MathBlock(MathBlock<'a>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Quote<'a>(Vec<Node<'a>>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Search<'a> {
    pub query: &'a str,
    pub content: &'a str,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CodeBlock<'a> {
    pub code: &'a str,
    pub lang: Option<&'a str>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MathBlock<'a> {
    pub formula: &'a str,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Inline<'a> {
    UnicodeEmoji(UnicodeEmoji<'a>),
    EmojiCode(EmojiCode<'a>),
    Bold(Bold<'a>),
    Small(Small<'a>),
    Italic(Italic<'a>),
    Strike(Strike<'a>),
    InlineCode(InlineCode<'a>),
    MathInline(MathInline<'a>),
    Mention(Mention<'a>),
    Hashtag(Hashtag<'a>),
    Url(Url<'a>),
    Link(Link<'a>),
    Fn(Fn<'a>),
    Plain(Plain<'a>),
    Text(Text<'a>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Simple<'a> {
    UnicodeEmoji(UnicodeEmoji<'a>),
    EmojiCode(EmojiCode<'a>),
    Text(Text<'a>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UnicodeEmoji<'a> {
    pub emoji: &'a str,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EmojiCode<'a> {
    pub name: &'a str,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Bold<'a>(Vec<Inline<'a>>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Small<'a>(Vec<Inline<'a>>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Italic<'a>(Vec<Inline<'a>>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Strike<'a>(Vec<Inline<'a>>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InlineCode<'a> {
    pub code: &'a str,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MathInline<'a> {
    pub formula: &'a str,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Mention<'a> {
    pub username: &'a str,
    pub host: Option<&'a str>,
    pub acct: &'a str,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Hashtag<'a> {
    pub hashtag: &'a str,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Url<'a> {
    pub url: &'a str,
    pub brackets: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Link<'a> {
    pub url: &'a str,
    pub silent: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Fn<'a> {
    pub name: &'a str,
    pub args: Vec<(&'a str, Option<&'a str>)>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Plain<'a>(Vec<Text<'a>>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Text<'a> {
    pub text: &'a str,
}
