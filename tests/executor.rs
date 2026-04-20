use std::collections::HashMap;

use buyan::{
    error::CompileError,
    stages::parse::{ast::ASTProgram, lexer::TokenKind},
};
use logos::Logos;

#[derive(Default)]
pub struct TestExecutor {
    input: String,
    imports: HashMap<String, String>,
    lex_result: Option<Result<Vec<TokenKind>, Vec<CompileError>>>,
    parse_result: Option<Result<ASTProgram, Vec<CompileError>>>,
}

impl TestExecutor {
    pub fn input<T: ToString>(input: T) -> Self {
        Self {
            input: input.to_string(),
            ..Default::default()
        }
    }

    pub fn lex(mut self) -> Self {
        assert!(self.lex_result.is_none());
        let mut result = Vec::new();
        let mut errors = Vec::new();
        for token in TokenKind::lexer(&self.input) {
            match token {
                Ok(token) => {
                    result.push(token);
                }
                Err(err) => {
                    errors.push(err);
                }
            }
        }
        if !errors.is_empty() {
            self.lex_result = Some(Err(errors));
            return self;
        }
        self.lex_result = Some(Ok(result));
        self
    }

    pub fn lex_ok(self) -> Self {
        assert!(self.lex_result.is_some());
        if let Some(result) = self.lex_result.as_ref() {
            assert!(
                result.is_ok(),
                "lex failed: {:?}",
                result.as_ref().unwrap_err()
            );
        }
        self
    }

    pub fn lex_match(self, expected: Result<Vec<TokenKind>, Vec<CompileError>>) -> Self {
        assert!(self.lex_result.is_some());
        assert_eq!(self.lex_result, Some(expected));
        self
    }

    pub fn parse(mut self) -> Self {
        assert!(self.parse_result.is_none());

        if self.lex_result.is_none() {
            self = self.lex();
        }

        self
    }

    pub fn parse_ok(self) -> Self {
        assert!(self.parse_result.is_some());
        if let Some(result) = self.parse_result.as_ref() {
            assert!(
                result.is_ok(),
                "parse failed: {:?}",
                result.as_ref().unwrap_err()
            );
        }
        self
    }
}
