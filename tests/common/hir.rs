#![allow(dead_code)]

use buyan::error::CompileError;

use crate::common::executor::TestExecutor;

impl TestExecutor {
    pub fn hir_ok(&self) -> bool {
        self.hir.as_ref().map(|hir| hir.is_ok()).unwrap_or(false)
    }

    pub fn assert_hir_ok(self) -> Self {
        assert!(self.hir_ok(), "hir stage failed {:?}", self.hir);
        self
    }

    pub fn assert_hir_err(self, pred: impl Fn(&CompileError) -> bool) -> Self {
        assert!(self.hir.is_some());
        assert!(self.hir.as_ref().unwrap().is_err());
        assert!(
            self.hir
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap_err()
                .iter()
                .any(pred),
            "error in hir not found {:?}",
            self.hir
        );
        self
    }
}
