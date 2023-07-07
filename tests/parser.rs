use mfm::node::*;

mod simple_parser {
    use super::*;

    mod text {
        use super::*;

        #[test]
        fn basic() {
            let input = "abc";
            let output = vec![Simple::Text(Text {
                text: "abc".to_string(),
            })];
            assert_eq!(mfm::parse_simple(input).unwrap(), output);
        }

        #[test]
        fn ignore_hashtag() {
            let input = "abc#abc";
            let output = vec![Simple::Text(Text {
                text: "abc#abc".to_string(),
            })];
            assert_eq!(mfm::parse_simple(input).unwrap(), output);
        }

        #[test]
        #[ignore]
        fn keycap_number_sign() {
            let input = "abc#️⃣abc";
            let output = vec![
                Simple::Text(Text {
                    text: "abc".to_string(),
                }),
                Simple::UnicodeEmoji(UnicodeEmoji {
                    emoji: "#️⃣".to_string(),
                }),
                Simple::Text(Text {
                    text: "abc".to_string(),
                }),
            ];
            assert_eq!(mfm::parse_simple(input).unwrap(), output);
        }
    }

    mod emoji {
        use super::*;

        #[test]
        fn basic() {
            let input = ":foo:";
            let output = vec![Simple::EmojiCode(EmojiCode {
                name: "foo".to_string(),
            })];
            assert_eq!(mfm::parse_simple(input).unwrap(), output);
        }

        #[test]
        fn between_texts() {
            let input = "foo:bar:baz";
            let output = vec![Simple::Text(Text {
                text: "foo:bar:baz".to_string(),
            })];
            assert_eq!(mfm::parse_simple(input).unwrap(), output);
        }

        #[test]
        fn between_texts_2() {
            let input = "12:34:56";
            let output = vec![Simple::Text(Text {
                text: "12:34:56".to_string(),
            })];
            assert_eq!(mfm::parse_simple(input).unwrap(), output);
        }

        #[test]
        fn between_texts_3() {
            let input = "あ:bar:い";
            let output = vec![
                Simple::Text(Text {
                    text: "あ".to_string(),
                }),
                Simple::EmojiCode(EmojiCode {
                    name: "bar".to_string(),
                }),
                Simple::Text(Text {
                    text: "い".to_string(),
                }),
            ];
            assert_eq!(mfm::parse_simple(input).unwrap(), output);
        }
    }

    #[test]
    fn disallow_other_syntaxes() {
        let input = "foo **bar** baz";
        let output = vec![Simple::Text(Text {
            text: "foo **bar** baz".to_string(),
        })];
        assert_eq!(mfm::parse_simple(input).unwrap(), output);
    }
}

mod full_parser {
    use super::*;

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

    mod search {
        use super::*;

        #[test]
        fn basic() {
            let input = "MFM 書き方 123 Search";
            let output = vec![Node::Block(Block::Search(Search {
                query: "MFM 書き方 123".to_string(),
                content: input.to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "MFM 書き方 123 [Search]";
            let output = vec![Node::Block(Block::Search(Search {
                query: "MFM 書き方 123".to_string(),
                content: input.to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "MFM 書き方 123 search";
            let output = vec![Node::Block(Block::Search(Search {
                query: "MFM 書き方 123".to_string(),
                content: input.to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "MFM 書き方 123 [search]";
            let output = vec![Node::Block(Block::Search(Search {
                query: "MFM 書き方 123".to_string(),
                content: input.to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "MFM 書き方 123 検索";
            let output = vec![Node::Block(Block::Search(Search {
                query: "MFM 書き方 123".to_string(),
                content: input.to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "MFM 書き方 123 [検索]";
            let output = vec![Node::Block(Block::Search(Search {
                query: "MFM 書き方 123".to_string(),
                content: input.to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn text_around_block() {
            let input = "abc\nhoge piyo bebeyo 検索\n123";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "abc".to_string(),
                })),
                Node::Block(Block::Search(Search {
                    query: "hoge piyo bebeyo".to_string(),
                    content: "hoge piyo bebeyo 検索".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "123".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }
    }

    mod code_block {
        use super::*;

        #[test]
        fn simple() {
            let input = "```\nabc\n```";
            let output = vec![Node::Block(Block::CodeBlock(CodeBlock {
                code: "abc".to_string(),
                lang: None,
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn multiple_line() {
            let input = "```\na\nb\nc\n```";
            let output = vec![Node::Block(Block::CodeBlock(CodeBlock {
                code: "a\nb\nc".to_string(),
                lang: None,
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn lang() {
            let input = "```js\nconst a = 1;\n```";
            let output = vec![Node::Block(Block::CodeBlock(CodeBlock {
                code: "const a = 1;".to_string(),
                lang: Some("js".to_string()),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn text_around_block() {
            let input = "abc\n```\nconst abc = 1;\n```\n123";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "abc".to_string(),
                })),
                Node::Block(Block::CodeBlock(CodeBlock {
                    code: "const abc = 1;".to_string(),
                    lang: None,
                })),
                Node::Inline(Inline::Text(Text {
                    text: "123".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_internal_marker() {
            let input = "```\naaa```bbb\n```";
            let output = vec![Node::Block(Block::CodeBlock(CodeBlock {
                code: "aaa```bbb".to_string(),
                lang: None,
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn trim_after_line_break() {
            let input = "```\nfoo\n```\nbar";
            let output = vec![
                Node::Block(Block::CodeBlock(CodeBlock {
                    code: "foo".to_string(),
                    lang: None,
                })),
                Node::Inline(Inline::Text(Text {
                    text: "bar".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }
    }

    mod math_block {
        use super::*;

        #[test]
        fn oneline() {
            let input = "\\[math1\\]";
            let output = vec![Node::Block(Block::MathBlock(MathBlock {
                formula: "math1".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn text_around_block() {
            let input = "abc\n\\[math1\\]\n123";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "abc".to_string(),
                })),
                Node::Block(Block::MathBlock(MathBlock {
                    formula: "math1".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "123".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn close_tag_not_on_line_ending() {
            let input = "\\[aaa\\]after";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "\\[aaa\\]after".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        #[ignore]
        fn open_tag_not_on_line_beginning() {
            let input = "before\\[aaa\\]";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "before\\[aaa\\]".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }
    }

    mod center {
        use super::*;

        #[test]
        fn single_text() {
            let input = "<center>abc</center>";
            let output = vec![Node::Block(Block::Center(Center(vec![Inline::Text(
                Text {
                    text: "abc".to_string(),
                },
            )])))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn multiple_text() {
            let input = "before\n<center>\nabc\n123\n\npiyo\n</center>\nafter";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "before".to_string(),
                })),
                Node::Block(Block::Center(Center(vec![Inline::Text(Text {
                    text: "abc\n123\n\npiyo".to_string(),
                })]))),
                Node::Inline(Inline::Text(Text {
                    text: "after".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }
    }

    mod emoji_code {
        use super::*;

        #[test]
        fn basic() {
            let input = ":abc:";
            let output = vec![Node::Inline(Inline::EmojiCode(EmojiCode {
                name: "abc".to_string(),
            }))];
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
}
