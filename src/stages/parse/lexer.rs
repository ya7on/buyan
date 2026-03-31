use std::path::PathBuf;

use chumsky::span::SimpleSpan;
use logos::Logos;

use crate::{common::Span, error::CompileError};

pub struct LexInput {
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug)]
pub struct LexResult {
    pub tokens: Vec<(TokenKind, SimpleSpan)>,
}

pub fn lex(input: LexInput) -> Result<LexResult, Vec<CompileError>> {
    let mut tokens = Vec::new();
    for (token, span) in TokenKind::lexer(&input.content).spanned() {
        tokens.push((token.map_err(|err| vec![err])?, SimpleSpan::from(span)));
    }
    Ok(LexResult { tokens })
}

fn map_err(err: &mut logos::Lexer<TokenKind>) -> CompileError {
    let span = err.span();
    CompileError::UnexpectedToken {
        span: Span {
            start: span.start,
            end: span.end,
        },
    }
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error(CompileError, map_err))]
pub enum TokenKind {
    #[regex(r"[ \t\n\r]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip, allow_greedy = true)]
    _Skip,

    /// Word
    #[regex(r"[a-zA-Z_]+[a-zA-Z_0-9]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex("[0-9]+u8", |lex| lex.slice()[..lex.slice().len()-2].parse::<u8>().ok())]
    #[regex(r"0[xX][0-9a-fA-F]+u8", |lex| u8::from_str_radix(&lex.slice()[2..lex.slice().len()-2], 16).ok())]
    LiteralU8(u8),
    #[regex(r#"\"[^\""]*\""#, |lex| {
            let s = lex.slice();
            s[1..s.len() - 1].to_string()
        })]
    LiteralString(String),

    #[token("import")]
    KeywordImport,
    #[token("module")]
    KeywordModule,
    #[token("def")]
    KeywordDef,
    #[token("end")]
    KeywordEnd,

    #[token("#")]
    Hash,
    #[token("+")]
    Plus,
    #[token(";")]
    Semicolon,
    #[token("(")]
    LeftParenthesis,
    #[token(")")]
    RightParenthesis,
    #[token("--")]
    MinusMinus,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token("[")]
    LeftSquareBracket,
    #[token("]")]
    RightSquareBracket,
    #[token("<")]
    LessThan,
    #[token(">")]
    GreaterThan,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("...")]
    Ellipsis,
    #[token(":")]
    Colon,
    #[token("|")]
    Pipe,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Result<Vec<TokenKind>, CompileError> {
        TokenKind::lexer(input).collect()
    }

    #[test]
    fn test_all_tokens() {
        let input = r#"import module def end ident 42u8 0x2Au8 "hello" # + ; ( ) -- . , [ ] < > { } ... : |"#;

        let tokens = lex(input).unwrap();

        assert_eq!(
            tokens,
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
            ]
        );
    }
}
