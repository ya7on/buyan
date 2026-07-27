#![allow(dead_code)]

use buyan::error::{DiagnosticKind, DiagnosticMessage};

use crate::common::executor::TestExecutor;

impl TestExecutor {
    pub fn parse_ok(&self) -> bool {
        self.ast.as_ref().is_some_and(Result::is_ok)
    }

    pub fn assert_parse_ok(self) -> Self {
        assert!(self.parse_ok(), "parse stage failed {:?}", self.ast);
        self
    }

    pub fn assert_parse_err(self, pred: impl Fn(&DiagnosticMessage) -> bool) -> Self {
        let Some(Err(diagnostics)) = &self.ast else {
            panic!("parse stage did not fail {:?}", self.ast);
        };
        assert!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.kind == DiagnosticKind::Error)
                .map(|diagnostic| &diagnostic.message)
                .any(pred),
            "error in ast not found {:?}",
            self.ast
        );
        self
    }
}
