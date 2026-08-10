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
        struct Inner(u8, u16);
        struct Outer(Inner);
        def main( -- u16)
            1u8 2u16 Inner< Outer<
            Outer.0 Inner.1
        end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok()
    .assert_ir_ok();
}
