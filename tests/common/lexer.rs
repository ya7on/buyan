#![allow(dead_code)]
use std::collections::HashMap;

use buyan::{
    error::CompileError,
    stages::parse::lexer::{LexInput, TokenKind, lex},
};

use crate::common::executor::TestExecutor;

impl TestExecutor {
    pub fn lex(mut self) -> Self {
        assert!(self.lex_result.is_none());
        let mut result = HashMap::new();
        for (path, content) in self.inputs.iter() {
            let lex_result = lex(LexInput {
                content: content.clone(),
            })
            .map(|result| result.tokens);
            result.insert(path.clone(), lex_result);
        }
        self.lex_result = Some(result);
        self
    }

    pub fn lex_ok(&self) -> bool {
        self.lex_result
            .as_ref()
            .unwrap()
            .iter()
            .all(|(_, r)| r.is_ok())
    }

    pub fn assert_lex_ok(self) -> Self {
        assert!(self.lex_result.is_some());
        for (path, result) in self.lex_result.as_ref().unwrap().iter() {
            assert!(result.is_ok(), "lex failed for path {} {:?}", path, result);
        }
        self
    }

    pub fn match_tokens<P: ToString>(self, path: P, expected: Vec<TokenKind>) -> Self {
        assert!(self.lex_result.is_some());
        let Some(result) = self.lex_result.as_ref().unwrap().get(&path.to_string()) else {
            panic!("no lex result for file {}", path.to_string())
        };
        assert!(result.is_ok());
        let tokens = result
            .clone()
            .unwrap()
            .iter()
            .map(|(kind, _)| kind.clone())
            .collect::<Vec<_>>();
        assert_eq!(tokens, expected);
        self
    }

    pub fn match_lex_err<P: ToString>(self, path: P, expected: Vec<CompileError>) -> Self {
        assert!(self.lex_result.is_some());
        let Some(result) = self.lex_result.as_ref().unwrap().get(&path.to_string()) else {
            panic!("no lex result for file {}", path.to_string())
        };
        assert!(result.is_err());
        let errors = result.clone().unwrap_err();
        assert_eq!(errors, expected);
        self
    }
}
