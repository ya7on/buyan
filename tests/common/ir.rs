#![allow(dead_code)]

use crate::common::executor::TestExecutor;

impl TestExecutor {
    pub fn assert_ir_ok(self) -> Self {
        self.ir.as_ref().map(|ir| ir.is_ok()).unwrap_or(false);
        self
    }
}
