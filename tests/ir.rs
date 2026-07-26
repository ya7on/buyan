use crate::common::executor::TestExecutor;

mod common;

#[test]
fn program() {
    TestExecutor::input(("app.by", "module app; def main( -- u8) 1u8 end"))
        .check()
        .assert_parse_ok()
        .assert_hir_ok()
        .assert_ir_ok();
}

#[test]
fn structs() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        struct Inner(u8, string);
        struct Outer(Inner);
        def main( -- string)
            1u8 "x" >Inner >Outer
            Outer.0 Inner.1
        end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok()
    .assert_ir_ok();
}
