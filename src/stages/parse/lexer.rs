use chumsky::span::SimpleSpan;
use logos::Logos;

use crate::{common::Span, error::CompileError};

pub struct LexInput {
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

fn unescape_string_literal(raw: &str) -> String {
    let mut result = String::new();
    let mut chars = raw[1..raw.len() - 1].chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }

        let Some(escaped) = chars.next() else {
            result.push('\\');
            break;
        };

        match escaped {
            'n' => result.push('\n'),
            'r' => result.push('\r'),
            't' => result.push('\t'),
            '\\' => result.push('\\'),
            '"' => result.push('"'),
            other => {
                result.push('\\');
                result.push(other);
            }
        }
    }

    result
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
    #[regex(r#""([^"\\]|\\.)*""#, |lex| unescape_string_literal(lex.slice()))]
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
