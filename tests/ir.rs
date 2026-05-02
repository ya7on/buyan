use crate::common::executor::TestExecutor;

mod common;

#[test]
fn test_simple_program() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main( -- ) end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok()
    .assert_ir_ok();
}
