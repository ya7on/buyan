#![allow(dead_code)]

use buyan::error::CompileError;

use crate::common::executor::TestExecutor;

impl TestExecutor {
    pub fn parse_ok(&self) -> bool {
        self.ast.as_ref().map(|ast| ast.is_ok()).unwrap_or(false)
    }

    pub fn assert_parse_ok(self) -> Self {
        assert!(self.parse_ok(), "parse stage failed {:?}", self.ast);
        self
    }

    pub fn assert_parse_err(self, pred: impl Fn(&CompileError) -> bool) -> Self {
        assert!(self.ast.is_some());
        assert!(self.ast.as_ref().unwrap().is_err());
        assert!(
            self.ast
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap_err()
                .iter()
                .any(pred),
            "error in ast not found {:?}",
            self.ast
        );
        self
    }
}
