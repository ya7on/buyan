#![allow(dead_code)]

use crate::common::executor::TestExecutor;

impl TestExecutor {
    pub fn assert_ir_ok(self) -> Self {
        assert!(
            self.ir.as_ref().is_some_and(Result::is_ok),
            "ir stage failed {:?}",
            self.ir
        );
        self
    }
}
