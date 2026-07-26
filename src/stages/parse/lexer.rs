use chumsky::span::Span;
use logos::Logos;

use crate::{
    common::{SourceId, SourceSpan},
    error::DiagnosticMessage,
};

pub struct LexInput {
    pub content: String,
    pub source_id: SourceId,
}

#[derive(Debug)]
pub struct LexResult {
    pub tokens: Vec<(TokenKind, SourceSpan)>,
}

pub fn lex(input: LexInput) -> Result<LexResult, Vec<DiagnosticMessage>> {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    for (token, range) in TokenKind::lexer(&input.content).spanned() {
        let span = SourceSpan::new(input.source_id, range);
        match token {
            Ok(token) => tokens.push((token, span)),
            Err(_) => errors.push(DiagnosticMessage::UnexpectedToken { span: span.into() }),
        }
    }
    if errors.is_empty() {
        Ok(LexResult { tokens })
    } else {
        Err(errors)
    }
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

#[derive(Logos, Debug, PartialEq, Clone)]
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
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<usize>().ok())]
    LiteralNumber(usize),
    #[regex(r#""([^"\\]|\\.)*""#, |lex| unescape_string_literal(lex.slice()))]
    LiteralString(String),
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    LiteralBool(bool),

    #[token("import")]
    KeywordImport,
    #[token("module")]
    KeywordModule,
    #[token("def")]
    KeywordDef,
    #[token("struct")]
    KeywordStruct,
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
