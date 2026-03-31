use std::collections::HashMap;

use crate::{
    error::CompileError,
    stages::parse::{ast::ASTProgram, lexer::TokenKind},
};

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

    pub fn lex(&mut self) {
        assert!(self.lex_result.is_none());

        todo!()
    }

    pub fn lex_ok(&mut self) {
        assert!(self.lex_result.is_some());
        if let Some(result) = self.lex_result.as_ref() {
            assert!(
                result.is_ok(),
                "lex failed: {:?}",
                result.as_ref().unwrap_err()
            );
        }
    }

    pub fn parse(&mut self) {
        assert!(self.parse_result.is_none());

        if self.lex_result.is_none() {
            self.lex();
        }

        todo!()
    }

    pub fn parse_ok(&mut self) {
        assert!(self.parse_result.is_some());
        if let Some(result) = self.parse_result.as_ref() {
            assert!(
                result.is_ok(),
                "parse failed: {:?}",
                result.as_ref().unwrap_err()
            );
        }
    }
}
