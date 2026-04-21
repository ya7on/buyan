#![allow(dead_code)]
use std::collections::HashMap;

use buyan::stages::parse::parser::{ParserInput, parse};

use crate::common::executor::TestExecutor;

impl TestExecutor {
    pub fn parse(mut self) -> Self {
        assert!(self.lex_result.is_some());
        assert!(self.parse_result.is_none());
        let mut result = HashMap::new();
        for (path, lex_result) in self.lex_result.as_ref().unwrap().iter() {
            let parse_result = parse(ParserInput {
                tokens: lex_result.clone().unwrap(),
            });
            result.insert(path.clone(), parse_result.map(|r| r.ast));
        }
        self.parse_result = Some(result);
        self
    }

    pub fn parse_ok(&self) -> bool {
        self.parse_result
            .as_ref()
            .unwrap()
            .iter()
            .all(|(_, r)| r.is_ok())
    }

    pub fn assert_parse_ok(self) -> Self {
        assert!(self.parse_result.is_some());
        for (path, result) in self.parse_result.as_ref().unwrap().iter() {
            assert!(
                result.is_ok(),
                "parse failed for path {} {:?}",
                path,
                result
            );
        }
        self
    }
}
