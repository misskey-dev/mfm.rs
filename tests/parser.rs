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

        #[test]
        fn open_tag_not_on_line_beginning() {
            let input = "before> aaa";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "before> aaa".to_string(),
            }))];
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

        #[test]
        fn mark_not_on_line_ending() {
            let input = "```\naaa\n```after";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "```\naaa\n```after".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn mark_not_on_line_beginning() {
            let input = "before```\naaa\n```";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "before```\naaa\n```".to_string(),
            }))];
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

        #[test]
        fn close_tag_not_on_line_ending() {
            let input = "<center>aaa</center>after";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "<center>aaa</center>after".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn open_tag_not_on_line_beginning() {
            let input = "before<center>aaa</center>";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "before<center>aaa</center>".to_string(),
            }))];
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

    mod big {
        use super::*;

        #[test]
        fn basic() {
            let input = "***abc***";
            let output = vec![Node::Inline(Inline::Fn(Fn {
                name: "tada".to_string(),
                args: Vec::new(),
                children: vec![Inline::Text(Text {
                    text: "abc".to_string(),
                })],
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn inline_contents() {
            let input = "***123**abc**123***";
            let output = vec![Node::Inline(Inline::Fn(Fn {
                name: "tada".to_string(),
                args: Vec::new(),
                children: vec![
                    Inline::Text(Text {
                        text: "123".to_string(),
                    }),
                    Inline::Bold(Bold(vec![Inline::Text(Text {
                        text: "abc".to_string(),
                    })])),
                    Inline::Text(Text {
                        text: "123".to_string(),
                    }),
                ],
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn multiple_line_contents() {
            let input = "***123\n**abc**\n123***";
            let output = vec![Node::Inline(Inline::Fn(Fn {
                name: "tada".to_string(),
                args: Vec::new(),
                children: vec![
                    Inline::Text(Text {
                        text: "123\n".to_string(),
                    }),
                    Inline::Bold(Bold(vec![Inline::Text(Text {
                        text: "abc".to_string(),
                    })])),
                    Inline::Text(Text {
                        text: "\n123".to_string(),
                    }),
                ],
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }
    }

    mod bold {
        use super::*;

        mod asta {
            use super::*;

            #[test]
            fn basic() {
                let input = "**abc**";
                let output = vec![Node::Inline(Inline::Bold(Bold(vec![Inline::Text(Text {
                    text: "abc".to_string(),
                })])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn inline_contents() {
                let input = "**123~~abc~~123**";
                let output = vec![Node::Inline(Inline::Bold(Bold(vec![
                    Inline::Text(Text {
                        text: "123".to_string(),
                    }),
                    Inline::Strike(Strike(vec![Inline::Text(Text {
                        text: "abc".to_string(),
                    })])),
                    Inline::Text(Text {
                        text: "123".to_string(),
                    }),
                ])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn multiple_line_contents() {
                let input = "**123\n~~abc~~\n123**";
                let output = vec![Node::Inline(Inline::Bold(Bold(vec![
                    Inline::Text(Text {
                        text: "123\n".to_string(),
                    }),
                    Inline::Strike(Strike(vec![Inline::Text(Text {
                        text: "abc".to_string(),
                    })])),
                    Inline::Text(Text {
                        text: "\n123".to_string(),
                    }),
                ])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }
        }

        mod tag {
            use super::*;

            #[test]
            fn basic() {
                let input = "<b>abc</b>";
                let output = vec![Node::Inline(Inline::Bold(Bold(vec![Inline::Text(Text {
                    text: "abc".to_string(),
                })])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn inline_contents() {
                let input = "<b>123~~abc~~123</b>";
                let output = vec![Node::Inline(Inline::Bold(Bold(vec![
                    Inline::Text(Text {
                        text: "123".to_string(),
                    }),
                    Inline::Strike(Strike(vec![Inline::Text(Text {
                        text: "abc".to_string(),
                    })])),
                    Inline::Text(Text {
                        text: "123".to_string(),
                    }),
                ])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn multiple_line_contents() {
                let input = "<b>123\n~~abc~~\n123</b>";
                let output = vec![Node::Inline(Inline::Bold(Bold(vec![
                    Inline::Text(Text {
                        text: "123\n".to_string(),
                    }),
                    Inline::Strike(Strike(vec![Inline::Text(Text {
                        text: "abc".to_string(),
                    })])),
                    Inline::Text(Text {
                        text: "\n123".to_string(),
                    }),
                ])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }
        }

        mod under {
            use super::*;

            #[test]
            fn basic() {
                let input = "__abc 123__";
                let output = vec![Node::Inline(Inline::Bold(Bold(vec![Inline::Text(Text {
                    text: "abc 123".to_string(),
                })])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn non_ascii() {
                let input = "__あ__";
                let output = vec![Node::Inline(Inline::Text(Text {
                    text: "__あ__".to_string(),
                }))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }
        }
    }

    mod small {
        use super::*;

        #[test]
        fn basic() {
            let input = "<small>abc</small>";
            let output = vec![Node::Inline(Inline::Small(Small(vec![Inline::Text(
                Text {
                    text: "abc".to_string(),
                },
            )])))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn inline_contents() {
            let input = "<small>123**abc**123</small>";
            let output = vec![Node::Inline(Inline::Small(Small(vec![
                Inline::Text(Text {
                    text: "123".to_string(),
                }),
                Inline::Bold(Bold(vec![Inline::Text(Text {
                    text: "abc".to_string(),
                })])),
                Inline::Text(Text {
                    text: "123".to_string(),
                }),
            ])))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn multiple_line_contents() {
            let input = "<small>123\n**abc**\n123</small>";
            let output = vec![Node::Inline(Inline::Small(Small(vec![
                Inline::Text(Text {
                    text: "123\n".to_string(),
                }),
                Inline::Bold(Bold(vec![Inline::Text(Text {
                    text: "abc".to_string(),
                })])),
                Inline::Text(Text {
                    text: "\n123".to_string(),
                }),
            ])))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }
    }

    mod italic {
        use super::*;

        mod tag {
            use super::*;

            #[test]
            fn basic() {
                let input = "<i>abc</i>";
                let output = vec![Node::Inline(Inline::Italic(Italic(vec![Inline::Text(
                    Text {
                        text: "abc".to_string(),
                    },
                )])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn inline_contents() {
                let input = "<i>abc**123**abc</i>";
                let output = vec![Node::Inline(Inline::Italic(Italic(vec![
                    Inline::Text(Text {
                        text: "abc".to_string(),
                    }),
                    Inline::Bold(Bold(vec![Inline::Text(Text {
                        text: "123".to_string(),
                    })])),
                    Inline::Text(Text {
                        text: "abc".to_string(),
                    }),
                ])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn multiple_line_contents() {
                let input = "<i>abc\n**123**\nabc</i>";
                let output = vec![Node::Inline(Inline::Italic(Italic(vec![
                    Inline::Text(Text {
                        text: "abc\n".to_string(),
                    }),
                    Inline::Bold(Bold(vec![Inline::Text(Text {
                        text: "123".to_string(),
                    })])),
                    Inline::Text(Text {
                        text: "\nabc".to_string(),
                    }),
                ])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }
        }

        mod asta {
            use super::*;

            #[test]
            fn basic() {
                let input = "*abc*";
                let output = vec![Node::Inline(Inline::Italic(Italic(vec![Inline::Text(
                    Text {
                        text: "abc".to_string(),
                    },
                )])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn basic_2() {
                let input = "before *abc* after";
                let output = vec![
                    Node::Inline(Inline::Text(Text {
                        text: "before ".to_string(),
                    })),
                    Node::Inline(Inline::Italic(Italic(vec![Inline::Text(Text {
                        text: "abc".to_string(),
                    })]))),
                    Node::Inline(Inline::Text(Text {
                        text: " after".to_string(),
                    })),
                ];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn ignore_if_preceded_by_alnum() {
                let input = "before*abc*after";
                let output = vec![Node::Inline(Inline::Text(Text {
                    text: "before*abc*after".to_string(),
                }))];
                assert_eq!(mfm::parse(input).unwrap(), output);

                let input = "123*abc*123";
                let output = vec![Node::Inline(Inline::Text(Text {
                    text: "123*abc*123".to_string(),
                }))];
                assert_eq!(mfm::parse(input).unwrap(), output);

                let input = "あいう*abc*えお";
                let output = vec![
                    Node::Inline(Inline::Text(Text {
                        text: "あいう".to_string(),
                    })),
                    Node::Inline(Inline::Italic(Italic(vec![Inline::Text(Text {
                        text: "abc".to_string(),
                    })]))),
                    Node::Inline(Inline::Text(Text {
                        text: "えお".to_string(),
                    })),
                ];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }
        }

        mod under {
            use super::*;

            #[test]
            fn basic() {
                let input = "_abc_";
                let output = vec![Node::Inline(Inline::Italic(Italic(vec![Inline::Text(
                    Text {
                        text: "abc".to_string(),
                    },
                )])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn basic_2() {
                let input = "before _abc_ after";
                let output = vec![
                    Node::Inline(Inline::Text(Text {
                        text: "before ".to_string(),
                    })),
                    Node::Inline(Inline::Italic(Italic(vec![Inline::Text(Text {
                        text: "abc".to_string(),
                    })]))),
                    Node::Inline(Inline::Text(Text {
                        text: " after".to_string(),
                    })),
                ];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn ignore_if_preceded_by_alnum() {
                let input = "before_abc_after";
                let output = vec![Node::Inline(Inline::Text(Text {
                    text: "before_abc_after".to_string(),
                }))];
                assert_eq!(mfm::parse(input).unwrap(), output);

                let input = "123_abc_123";
                let output = vec![Node::Inline(Inline::Text(Text {
                    text: "123_abc_123".to_string(),
                }))];
                assert_eq!(mfm::parse(input).unwrap(), output);

                let input = "あいう_abc_えお";
                let output = vec![
                    Node::Inline(Inline::Text(Text {
                        text: "あいう".to_string(),
                    })),
                    Node::Inline(Inline::Italic(Italic(vec![Inline::Text(Text {
                        text: "abc".to_string(),
                    })]))),
                    Node::Inline(Inline::Text(Text {
                        text: "えお".to_string(),
                    })),
                ];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }
        }
    }

    mod strike {
        use super::*;

        mod tag {
            use super::*;

            #[test]
            fn basic() {
                let input = "<s>foo</s>";
                let output = vec![Node::Inline(Inline::Strike(Strike(vec![Inline::Text(
                    Text {
                        text: "foo".to_string(),
                    },
                )])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }
        }

        mod wave {
            use super::*;

            #[test]
            fn basic() {
                let input = "~~foo~~";
                let output = vec![Node::Inline(Inline::Strike(Strike(vec![Inline::Text(
                    Text {
                        text: "foo".to_string(),
                    },
                )])))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn newline_between_marks() {
                let input = "~~foo\nbar~~";
                let output = vec![Node::Inline(Inline::Text(Text {
                    text: "~~foo\nbar~~".to_string(),
                }))];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }
        }
    }

    mod inline_code {
        use super::*;

        #[test]
        fn basic() {
            let input = r#"`var x = "Strawberry Pasta";`"#;
            let output = vec![Node::Inline(Inline::InlineCode(InlineCode {
                code: r#"var x = "Strawberry Pasta";"#.to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn disallow_line_break() {
            let input = "`foo\nbar`";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "`foo\nbar`".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn disallow_acute_accent() {
            let input = "`foo´bar`";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "`foo´bar`".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }
    }

    mod math_inline {
        use super::*;

        #[test]
        fn basic() {
            let input = r"\(x = {-b \pm \sqrt{b^2-4ac} \over 2a}\)";
            let output = vec![Node::Inline(Inline::MathInline(MathInline {
                formula: r"x = {-b \pm \sqrt{b^2-4ac} \over 2a}".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }
    }

    mod mention {
        use super::*;

        #[test]
        fn basic() {
            let input = "@abc";
            let output = vec![Node::Inline(Inline::Mention(Mention {
                username: "abc".to_string(),
                host: None,
                acct: "@abc".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn basic_2() {
            let input = "before @abc after";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "before ".to_string(),
                })),
                Node::Inline(Inline::Mention(Mention {
                    username: "abc".to_string(),
                    host: None,
                    acct: "@abc".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: " after".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn basic_remote() {
            let input = "@abc@misskey.io";
            let output = vec![Node::Inline(Inline::Mention(Mention {
                username: "abc".to_string(),
                host: Some("misskey.io".to_string()),
                acct: "@abc@misskey.io".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn basic_remote_2() {
            let input = "before @abc@misskey.io after";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "before ".to_string(),
                })),
                Node::Inline(Inline::Mention(Mention {
                    username: "abc".to_string(),
                    host: Some("misskey.io".to_string()),
                    acct: "@abc@misskey.io".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: " after".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn basic_remote_3() {
            let input = "before\n@abc@misskey.io\nafter";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "before\n".to_string(),
                })),
                Node::Inline(Inline::Mention(Mention {
                    username: "abc".to_string(),
                    host: Some("misskey.io".to_string()),
                    acct: "@abc@misskey.io".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "\nafter".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_mail_address() {
            let input = "abc@example.com";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "abc@example.com".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn non_ascii_before_mention() {
            let input = "あいう@abc";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "あいう".to_string(),
                })),
                Node::Inline(Inline::Mention(Mention {
                    username: "abc".to_string(),
                    host: None,
                    acct: "@abc".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn invalid_char_only_username() {
            let input = "@-";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "@-".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "@-@aaa";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "@-@aaa".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn invalid_char_only_hostname() {
            let input = "@abc@.";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "@abc@.".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn allow_dash_in_username() {
            let input = "@abc-d";
            let output = vec![Node::Inline(Inline::Mention(Mention {
                username: "abc-d".to_string(),
                host: None,
                acct: "@abc-d".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn disallow_dash_in_head_of_username() {
            let input = "@-abc";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "@-abc".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "@-abc@aaa";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "@-abc@aaa".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn disallow_dash_in_tail_of_username() {
            let input = "@abc-";
            let output = vec![
                Node::Inline(Inline::Mention(Mention {
                    username: "abc".to_string(),
                    host: None,
                    acct: "@abc".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "-".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "@abc--";
            let output = vec![
                Node::Inline(Inline::Mention(Mention {
                    username: "abc".to_string(),
                    host: None,
                    acct: "@abc".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "--".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "@abc-@aaa";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "@abc-@aaa".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn disallow_dot_in_head_of_hostname() {
            let input = "@abc@.aaa";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "@abc@.aaa".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn disallow_dot_in_tail_of_hostname() {
            let input = "@abc@aaa.";
            let output = vec![
                Node::Inline(Inline::Mention(Mention {
                    username: "abc".to_string(),
                    host: Some("aaa".to_string()),
                    acct: "@abc@aaa".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: ".".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn disallow_dash_in_head_of_hostname() {
            let input = "@abc@-aaa";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "@abc@-aaa".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn disallow_dash_in_tail_of_hostname() {
            let input = "@abc@aaa-";
            let output = vec![
                Node::Inline(Inline::Mention(Mention {
                    username: "abc".to_string(),
                    host: Some("aaa".to_string()),
                    acct: "@abc@aaa".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "-".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "@abc@aaa.-";
            let output = vec![
                Node::Inline(Inline::Mention(Mention {
                    username: "abc".to_string(),
                    host: Some("aaa".to_string()),
                    acct: "@abc@aaa".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: ".-".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }
    }

    mod hashtag {
        use super::*;

        #[test]
        fn basic() {
            let input = "#abc";
            let output = vec![Node::Inline(Inline::Hashtag(Hashtag {
                hashtag: "abc".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn basic_2() {
            let input = "before #abc after";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "before ".to_string(),
                })),
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "abc".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: " after".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        #[ignore]
        fn with_keycap_number_sign() {
            let input = "#️⃣abc123 #abc";
            let output = vec![
                Node::Inline(Inline::UnicodeEmoji(UnicodeEmoji {
                    emoji: "#️⃣".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "abc123 ".to_string(),
                })),
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "abc".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        #[ignore]
        fn with_keycap_number_sign_2() {
            let input = "abc\n#️⃣abc";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "abc\n".to_string(),
                })),
                Node::Inline(Inline::UnicodeEmoji(UnicodeEmoji {
                    emoji: "#️⃣".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "abc".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn alnum_before_hashtag() {
            let input = "abc#abc";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "abc#abc".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "あいう#abc";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "あいう".to_string(),
                })),
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "abc".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_comma_and_period() {
            let input = "Foo #bar, baz #piyo.";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "Foo ".to_string(),
                })),
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "bar".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: ", baz ".to_string(),
                })),
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "piyo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: ".".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_exclamation_mark() {
            let input = "#Foo!";
            let output = vec![
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "Foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "!".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_colon() {
            let input = "#Foo:";
            let output = vec![
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "Foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: ":".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_single_quote() {
            let input = "#Foo'";
            let output = vec![
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "Foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "'".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_double_quote() {
            let input = "#Foo\"";
            let output = vec![
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "Foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "\"".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_square_bracket() {
            let input = "#Foo]";
            let output = vec![
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "Foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "]".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_slash() {
            let input = "#foo/bar";
            let output = vec![
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "/bar".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_angle_bracket() {
            let input = "#foo<bar>";
            let output = vec![
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "<bar>".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn allow_including_number() {
            let input = "#foo123";
            let output = vec![Node::Inline(Inline::Hashtag(Hashtag {
                hashtag: "foo123".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn with_parens() {
            let input = "(#foo)";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "(".to_string(),
                })),
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: ")".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "#(foo)";
            let output = vec![Node::Inline(Inline::Hashtag(Hashtag {
                hashtag: "(foo)".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn with_brackets() {
            let input = "[#foo]";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "[".to_string(),
                })),
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "]".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "#[foo]";
            let output = vec![Node::Inline(Inline::Hashtag(Hashtag {
                hashtag: "[foo]".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn with_fullwidth_parens() {
            let input = "（#foo）";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "（".to_string(),
                })),
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "）".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "#（foo）";
            let output = vec![Node::Inline(Inline::Hashtag(Hashtag {
                hashtag: "（foo）".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn with_corner_brackets() {
            let input = "「#foo」";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "「".to_string(),
                })),
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "」".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "#「foo」";
            let output = vec![Node::Inline(Inline::Hashtag(Hashtag {
                hashtag: "「foo」".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn with_mixed_brackets() {
            let input = "「#foo(bar)」";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "「".to_string(),
                })),
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "foo(bar)".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "」".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn with_parens_space_before() {
            let input = "(bar #foo)";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "(bar ".to_string(),
                })),
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: ")".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn with_square_brackets_space_before() {
            let input = "「bar #foo」";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "「bar ".to_string(),
                })),
                Node::Inline(Inline::Hashtag(Hashtag {
                    hashtag: "foo".to_string(),
                })),
                Node::Inline(Inline::Text(Text {
                    text: "」".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn disallow_number_only() {
            let input = "#123";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "#123".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn disallow_number_only_with_parens() {
            let input = "(#123)";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "(#123)".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }
    }

    mod url {
        use super::*;

        #[test]
        fn basic() {
            let input = "https://misskey.io/@ai";
            let output = vec![Node::Inline(Inline::Url(Url {
                url: "https://misskey.io/@ai".to_string(),
                brackets: false,
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn with_other_texts() {
            let input = "official instance: https://misskey.io/@ai.";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "official instance: ".to_string(),
                })),
                Node::Inline(Inline::Url(Url {
                    url: "https://misskey.io/@ai".to_string(),
                    brackets: false,
                })),
                Node::Inline(Inline::Text(Text {
                    text: ".".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_trailing_period() {
            let input = "https://misskey.io/@ai.";
            let output = vec![
                Node::Inline(Inline::Url(Url {
                    url: "https://misskey.io/@ai".to_string(),
                    brackets: false,
                })),
                Node::Inline(Inline::Text(Text {
                    text: ".".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn disallow_period_only() {
            let input = "https://.";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "https://.".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_trailing_periods() {
            let input = "https://misskey.io/@ai...";
            let output = vec![
                Node::Inline(Inline::Url(Url {
                    url: "https://misskey.io/@ai".to_string(),
                    brackets: false,
                })),
                Node::Inline(Inline::Text(Text {
                    text: "...".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn with_comma() {
            let input = "https://example.com/foo?bar=a,b";
            let output = vec![Node::Inline(Inline::Url(Url {
                url: "https://example.com/foo?bar=a,b".to_string(),
                brackets: false,
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_trailing_comma() {
            let input = "https://example.com/foo, bar";
            let output = vec![
                Node::Inline(Inline::Url(Url {
                    url: "https://example.com/foo".to_string(),
                    brackets: false,
                })),
                Node::Inline(Inline::Text(Text {
                    text: ", bar".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn with_brackets() {
            let input = "https://example.com/foo(bar)";
            let output = vec![Node::Inline(Inline::Url(Url {
                url: "https://example.com/foo(bar)".to_string(),
                brackets: false,
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);

            let input = "https://example.com/foo[bar]";
            let output = vec![Node::Inline(Inline::Url(Url {
                url: "https://example.com/foo[bar]".to_string(),
                brackets: false,
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_parent_parens() {
            let input = "(https://example.com/foo)";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "(".to_string(),
                })),
                Node::Inline(Inline::Url(Url {
                    url: "https://example.com/foo".to_string(),
                    brackets: false,
                })),
                Node::Inline(Inline::Text(Text {
                    text: ")".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_parent_parens_2() {
            let input = "(foo https://example.com/foo)";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "(foo ".to_string(),
                })),
                Node::Inline(Inline::Url(Url {
                    url: "https://example.com/foo".to_string(),
                    brackets: false,
                })),
                Node::Inline(Inline::Text(Text {
                    text: ")".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_parent_parens_with_internal_parens() {
            let input = "(https://example.com/foo(bar))";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "(".to_string(),
                })),
                Node::Inline(Inline::Url(Url {
                    url: "https://example.com/foo(bar)".to_string(),
                    brackets: false,
                })),
                Node::Inline(Inline::Text(Text {
                    text: ")".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_parent_brackets() {
            let input = "foo [https://example.com/foo] bar";
            let output = vec![
                Node::Inline(Inline::Text(Text {
                    text: "foo [".to_string(),
                })),
                Node::Inline(Inline::Url(Url {
                    url: "https://example.com/foo".to_string(),
                    brackets: false,
                })),
                Node::Inline(Inline::Text(Text {
                    text: "] bar".to_string(),
                })),
            ];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn ignore_non_ascii_without_angle_brackets() {
            let input = "https://大石泉すき.example.com";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "https://大石泉すき.example.com".to_string(),
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn match_non_ascii_with_angle_brackets() {
            let input = "<https://大石泉すき.example.com>";
            let output = vec![Node::Inline(Inline::Url(Url {
                url: "https://大石泉すき.example.com".to_string(),
                brackets: true,
            }))];
            assert_eq!(mfm::parse(input).unwrap(), output);
        }

        #[test]
        fn prevent_xss() {
            let input = "javascript:foo";
            let output = vec![Node::Inline(Inline::Text(Text {
                text: "javascript:foo".to_string(),
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

        #[test]
        fn big() {
            let input = "<b><b>***abc***</b></b>";
            let output = vec![Node::Inline(Inline::Bold(Bold(vec![Inline::Bold(Bold(
                vec![Inline::Text(Text {
                    text: "***abc***".to_string(),
                })],
            ))])))];
            assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);
        }

        mod bold {
            use super::*;

            #[test]
            fn basic() {
                let input = "<i><i>**abc**</i></i>";
                let output = vec![Node::Inline(Inline::Italic(Italic(vec![Inline::Italic(
                    Italic(vec![Inline::Text(Text {
                        text: "**abc**".to_string(),
                    })]),
                )])))];
                assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);
            }

            #[test]
            fn tag() {
                let input = "<i><i><b>abc</b></i></i>";
                let output = vec![Node::Inline(Inline::Italic(Italic(vec![Inline::Italic(
                    Italic(vec![Inline::Text(Text {
                        text: "<b>abc</b>".to_string(),
                    })]),
                )])))];
                assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);
            }
        }

        #[test]
        fn small() {
            let input = "<i><i><small>abc</small></i></i>";
            let output = vec![Node::Inline(Inline::Italic(Italic(vec![Inline::Italic(
                Italic(vec![Inline::Text(Text {
                    text: "<small>abc</small>".to_string(),
                })]),
            )])))];
            assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);
        }

        #[test]
        fn italic() {
            let input = "<b><b><i>abc</i></b></b>";
            let output = vec![Node::Inline(Inline::Bold(Bold(vec![Inline::Bold(Bold(
                vec![Inline::Text(Text {
                    text: "<i>abc</i>".to_string(),
                })],
            ))])))];
            assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);
        }

        mod strike {
            use super::*;

            #[test]
            fn basic() {
                let input = "<b><b>~~abc~~</b></b>";
                let output = vec![Node::Inline(Inline::Bold(Bold(vec![Inline::Bold(Bold(
                    vec![Inline::Text(Text {
                        text: "~~abc~~".to_string(),
                    })],
                ))])))];
                assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);
            }

            #[test]
            fn tag() {
                let input = "<b><b><s>abc</s></b></b>";
                let output = vec![Node::Inline(Inline::Bold(Bold(vec![Inline::Bold(Bold(
                    vec![Inline::Text(Text {
                        text: "<s>abc</s>".to_string(),
                    })],
                ))])))];
                assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);
            }
        }

        mod hashtag {
            use super::*;

            #[test]
            fn basic() {
                let input = "<b>#abc(xyz)</b>";
                let output = vec![Node::Inline(Inline::Bold(Bold(vec![Inline::Hashtag(
                    Hashtag {
                        hashtag: "abc(xyz)".to_string(),
                    },
                )])))];
                assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);

                let input = "<b>#abc(x(y)z)</b>";
                let output = vec![Node::Inline(Inline::Bold(Bold(vec![
                    Inline::Hashtag(Hashtag {
                        hashtag: "abc".to_string(),
                    }),
                    Inline::Text(Text {
                        text: "(x(y)z)".to_string(),
                    }),
                ])))];
                assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);
            }

            #[test]
            fn outside_parens() {
                let input = "(#abc)";
                let output = vec![
                    Node::Inline(Inline::Text(Text {
                        text: "(".to_string(),
                    })),
                    Node::Inline(Inline::Hashtag(Hashtag {
                        hashtag: "abc".to_string(),
                    })),
                    Node::Inline(Inline::Text(Text {
                        text: ")".to_string(),
                    })),
                ];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn outside_brackets() {
                let input = "[#abc]";
                let output = vec![
                    Node::Inline(Inline::Text(Text {
                        text: "[".to_string(),
                    })),
                    Node::Inline(Inline::Hashtag(Hashtag {
                        hashtag: "abc".to_string(),
                    })),
                    Node::Inline(Inline::Text(Text {
                        text: "]".to_string(),
                    })),
                ];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn outside_corner_brackets() {
                let input = "「#abc」";
                let output = vec![
                    Node::Inline(Inline::Text(Text {
                        text: "「".to_string(),
                    })),
                    Node::Inline(Inline::Hashtag(Hashtag {
                        hashtag: "abc".to_string(),
                    })),
                    Node::Inline(Inline::Text(Text {
                        text: "」".to_string(),
                    })),
                ];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }

            #[test]
            fn outside_fullwidth_parens() {
                let input = "（#abc）";
                let output = vec![
                    Node::Inline(Inline::Text(Text {
                        text: "（".to_string(),
                    })),
                    Node::Inline(Inline::Hashtag(Hashtag {
                        hashtag: "abc".to_string(),
                    })),
                    Node::Inline(Inline::Text(Text {
                        text: "）".to_string(),
                    })),
                ];
                assert_eq!(mfm::parse(input).unwrap(), output);
            }
        }

        #[test]
        fn url() {
            let input = "<b>https://example.com/abc(xyz)</b>";
            let output = vec![Node::Inline(Inline::Bold(Bold(vec![Inline::Url(Url {
                url: "https://example.com/abc(xyz)".to_string(),
                brackets: false,
            })])))];
            assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);

            let input = "<b>https://example.com/abc(x(y)z)</b>";
            let output = vec![Node::Inline(Inline::Bold(Bold(vec![
                Inline::Url(Url {
                    url: "https://example.com/abc".to_string(),
                    brackets: false,
                }),
                Inline::Text(Text {
                    text: "(x(y)z)".to_string(),
                }),
            ])))];
            assert_eq!(mfm::parse_with_nest_limit(input, 2).unwrap(), output);
        }
    }
}
