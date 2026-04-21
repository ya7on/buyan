#![allow(dead_code)]

use std::collections::HashMap;

use buyan::{
    error::CompileError,
    stages::parse::{ast::ASTModule, lexer::TokenKind},
};
use chumsky::span::SimpleSpan;

type LexerResult = Result<Vec<(TokenKind, SimpleSpan)>, Vec<CompileError>>;
type ParserResult = Result<ASTModule, Vec<CompileError>>;

#[derive(Default)]
pub struct TestExecutor {
    pub inputs: HashMap<String, String>,
    pub lex_result: Option<HashMap<String, LexerResult>>,
    pub parse_result: Option<HashMap<String, ParserResult>>,
}

impl TestExecutor {
    pub fn input<P: ToString, C: ToString>(input: (P, C)) -> Self {
        let (path, content) = input;
        Self {
            inputs: vec![(path.to_string(), content.to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        }
    }

    pub fn check(mut self) -> Self {
        self = self.lex();
        if !self.lex_ok() {
            return self;
        }
        self = self.parse();
        if !self.parse_ok() {
            return self;
        }
        self
    }
}
