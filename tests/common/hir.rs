#![allow(dead_code)]

use buyan::error::{DiagnosticKind, DiagnosticMessage};

use crate::common::executor::TestExecutor;

impl TestExecutor {
    pub fn hir_ok(&self) -> bool {
        self.hir.as_ref().is_some_and(Result::is_ok)
    }

    pub fn assert_hir_ok(self) -> Self {
        assert!(self.hir_ok(), "hir stage failed {:?}", self.hir);
        self
    }

    pub fn assert_hir_err(self, pred: impl Fn(&DiagnosticMessage) -> bool) -> Self {
        let Some(Err(diagnostics)) = &self.hir else {
            panic!("HIR stage did not fail {:?}", self.hir);
        };
        assert!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.kind == DiagnosticKind::Error)
                .map(|diagnostic| &diagnostic.message)
                .any(pred),
            "error in hir not found {:?}",
            self.hir
        );
        self
    }
}
