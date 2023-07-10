use mfm::node::*;

mod text {
    use super::*;

    #[test]
    fn basic() {
        let input = "abc";
        let output = vec![Node::Inline(Inline::Text(Text {
            text: "abc".to_string(),
        }))];
        assert_eq!(mfm::parse(input).unwrap(), output);
    }
}

mod quote {
    use super::*;

    #[test]
    fn single_line() {
        let input = "> abc";
        let output = vec![Node::Block(Block::Quote(Quote(vec![Node::Inline(
            Inline::Text(Text {
                text: "abc".to_string(),
            }),
        )])))];
        assert_eq!(mfm::parse(input).unwrap(), output);
    }

    #[test]
    fn multiple_line() {
        let input = r#"
> abc
> 123
"#;
        let output = vec![Node::Block(Block::Quote(Quote(vec![Node::Inline(
            Inline::Text(Text {
                text: "abc\n123".to_string(),
            }),
        )])))];
        assert_eq!(mfm::parse(input).unwrap(), output);
    }

    #[test]
    #[ignore]
    fn nest_block() {
        let input = r#"
> <center>
> a
> </center>
"#;
        let output = vec![Node::Block(Block::Quote(Quote(vec![Node::Block(
            Block::Center(Center(vec![Inline::Text(Text {
                text: "a".to_string(),
            })])),
        )])))];
        assert_eq!(mfm::parse(input).unwrap(), output);
    }

    #[test]
    #[ignore]
    fn nest_block_with_inline() {
        let input = r#"
> <center>
> I'm @ai, An bot of misskey!
> </center>
"#;
        let output = vec![Node::Block(Block::Quote(Quote(vec![Node::Block(
            Block::Center(Center(vec![
                Inline::Text(Text {
                    text: "I'm ".to_string(),
                }),
                Inline::Mention(Mention {
                    username: "ai".to_string(),
                    host: None,
                    acct: "@ai".to_string(),
                }),
                Inline::Text(Text {
                    text: ", An bot of misskey!".to_string(),
                }),
            ])),
        )])))];
        assert_eq!(mfm::parse(input).unwrap(), output);
    }

    #[test]
    fn multiple_line_with_empty_line() {
        let input = r#"
> abc
>
> 123
"#;
        let output = vec![Node::Block(Block::Quote(Quote(vec![Node::Inline(
            Inline::Text(Text {
                text: "abc\n\n123".to_string(),
            }),
        )])))];
        assert_eq!(mfm::parse(input).unwrap(), output);
    }

    #[test]
    fn single_empty_line() {
        let input = "> ";
        let output = vec![Node::Inline(Inline::Text(Text {
            text: "> ".to_string(),
        }))];
        assert_eq!(mfm::parse(input).unwrap(), output);
    }

    #[test]
    fn ignore_empty_line_after_quote() {
        let input = r#"
> foo
> bar

hoge"#;
        let output = vec![
            Node::Block(Block::Quote(Quote(vec![Node::Inline(Inline::Text(
                Text {
                    text: "foo\nbar".to_string(),
                },
            ))]))),
            Node::Inline(Inline::Text(Text {
                text: "hoge".to_string(),
            })),
        ];
        assert_eq!(mfm::parse(input).unwrap(), output);
    }

    #[test]
    fn two_quote_blocks() {
        let input = r#"
> foo

> bar

hoge"#;
        let output = vec![
            Node::Block(Block::Quote(Quote(vec![Node::Inline(Inline::Text(
                Text {
                    text: "foo".to_string(),
                },
            ))]))),
            Node::Block(Block::Quote(Quote(vec![Node::Inline(Inline::Text(
                Text {
                    text: "bar".to_string(),
                },
            ))]))),
            Node::Inline(Inline::Text(Text {
                text: "hoge".to_string(),
            })),
        ];
        assert_eq!(mfm::parse(input).unwrap(), output);
    }
}

mod plain {
    use super::*;

    #[test]
    fn multiple_line() {
        let input = "a\n<plain>\n**Hello**\nworld\n</plain>\nb";
        let output = vec![
            Node::Inline(Inline::Text(Text {
                text: "a\n".to_string(),
            })),
            Node::Inline(Inline::Plain(Plain(vec![Text {
                text: "**Hello**\nworld".to_string(),
            }]))),
            Node::Inline(Inline::Text(Text {
                text: "\nb".to_string(),
            })),
        ];
        assert_eq!(mfm::parse(input).unwrap(), output)
    }

    #[test]
    fn single_line() {
        let input = "a\n<plain>\n**Hello** world\n</plain>\nb";
        let output = vec![
            Node::Inline(Inline::Text(Text {
                text: "a\n".to_string(),
            })),
            Node::Inline(Inline::Plain(Plain(vec![Text {
                text: "**Hello** world".to_string(),
            }]))),
            Node::Inline(Inline::Text(Text {
                text: "\nb".to_string(),
            })),
        ];
        assert_eq!(mfm::parse(input).unwrap(), output)
    }
}

mod nesting_limit {
    use super::*;

    mod quote {
        use super::*;

        #[test]
        fn basic() {
            let input = ">>> abc";
            let output = vec![Node::Block(Block::Quote(Quote(vec![Node::Block(
                Block::Quote(Quote(vec![Node::Inline(Inline::Text(Text {
                    text: "> abc".to_string(),
                }))])),
            )])))];
            assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);
        }

        #[test]
        fn basic2() {
            let input = ">> **abc**";
            let output = vec![Node::Block(Block::Quote(Quote(vec![Node::Block(
                Block::Quote(Quote(vec![Node::Inline(Inline::Text(Text {
                    text: "**abc**".to_string(),
                }))])),
            )])))];
            assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);
        }
    }
}
