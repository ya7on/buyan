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

#[test]
fn test_struct_lowering() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        struct Pair(u8, string);
        def main( -- u8, string)
            7u8 "seven" >Pair Pair>
        end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok()
    .assert_ir_ok();
}

#[test]
fn test_empty_and_nested_struct_lowering() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        struct Unit();
        struct Inner(u8);
        struct Outer(Inner, Unit);
        def main( -- u8)
            9u8 >Inner >Unit >Outer
            Outer> Unit> Inner>
        end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok()
    .assert_ir_ok();
}
