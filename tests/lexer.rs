use buyan::{common::Span, error::CompileError, stages::parse::lexer::TokenKind};

use crate::common::executor::TestExecutor;

mod common;

#[test]
fn test_lexer_all_tokens() {
    TestExecutor::input((
        "app",
        r#"import module def end ident 42u8 0x2Au8 "hello" # + ; ( ) -- . , [ ] < > { } ... : |"#,
    ))
    .check()
    .assert_lex_ok()
    .match_tokens(
        "app",
        vec![
            TokenKind::KeywordImport,
            TokenKind::KeywordModule,
            TokenKind::KeywordDef,
            TokenKind::KeywordEnd,
            TokenKind::Ident("ident".to_string()),
            TokenKind::LiteralU8(42),
            TokenKind::LiteralU8(42),
            TokenKind::LiteralString("hello".to_string()),
            TokenKind::Hash,
            TokenKind::Plus,
            TokenKind::Semicolon,
            TokenKind::LeftParenthesis,
            TokenKind::RightParenthesis,
            TokenKind::MinusMinus,
            TokenKind::Dot,
            TokenKind::Comma,
            TokenKind::LeftSquareBracket,
            TokenKind::RightSquareBracket,
            TokenKind::LessThan,
            TokenKind::GreaterThan,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Ellipsis,
            TokenKind::Colon,
            TokenKind::Pipe,
        ],
    );
}

#[test]
fn test_unknown_token() {
    TestExecutor::input(("app", r#"!"#)).check().match_lex_err(
        "app",
        vec![CompileError::UnexpectedToken {
            span: Span { start: 0, end: 1 },
        }],
    );
}
