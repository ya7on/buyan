use buyan::error::DiagnosticMessage;

use crate::common::executor::TestExecutor;

mod common;

#[test]
fn program() {
    TestExecutor::input((
        "app.by",
        r#"
        import dep;
        import deep.lib;
        import std.stack;
        module app;
        #[intrinsic]
        def main( -- u8) 1u8 dep.word end
        "#,
    ))
    .add_file(("dep.by", "module dep; def word( -- ) end"))
    .add_file(("deep/lib.by", "module deep.lib;"))
    .check()
    .assert_parse_ok();
}

#[test]
fn generics() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def swap<...S, A, B>(...S, A, B -- ...S, B, A) end
        "#,
    ))
    .check()
    .assert_parse_ok();
}

#[test]
fn structs() {
    TestExecutor::input((
        "app.by",
        r#"
        import geometry;
        module app;
        struct Pair(u8, bool);
        def unpack(Pair -- u8, bool) Pair> end
        def field(Pair -- u8) Pair.0 end
        def qualified( -- geometry.Point) 1u8 2u8 >geometry.Point end
        def path( -- ) geometry.Point.0.tail end
        "#,
    ))
    .add_file(("geometry.by", "module geometry; struct Point(u8, u8);"))
    .check()
    .assert_parse_ok();
}

#[test]
fn lambdas() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def apply(|u8 -- u8| -- ) end
        def main( -- ) |u8 -- u8| { 1u8 } apply end
        "#,
    ))
    .check()
    .assert_parse_ok();
}

#[test]
fn token() {
    TestExecutor::input(("app.by", "!"))
        .check()
        .assert_parse_err(|error| matches!(error, DiagnosticMessage::UnexpectedToken { .. }));
}

#[test]
fn syntax() {
    TestExecutor::input(("app.by", "module app; def main( -- ) end struct A(u8);"))
        .check()
        .assert_parse_err(|error| matches!(error, DiagnosticMessage::ParseError { .. }));
}

#[test]
fn import() {
    TestExecutor::input(("app.by", "import missing; module app;"))
        .check()
        .assert_parse_err(|error| matches!(error, DiagnosticMessage::ImportError { .. }));
}
