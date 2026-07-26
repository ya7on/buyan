use buyan::error::CompileError;

use crate::common::executor::TestExecutor;

mod common;

#[test]
fn values() {
    TestExecutor::input((
        "app.by",
        r#"
        import std.math;
        module app;
        def main( -- u8, string) 1u8 2u8 std.math.add "x" end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}

#[test]
fn generics() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def id<A>(A -- A) end
        def keep<...S, A>(...S, A -- ...S, A) end
        def main( -- u8) 1u8 id keep end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}

#[test]
fn lambdas() {
    TestExecutor::input((
        "app.by",
        r#"
        import std.cfg;
        import std.math;
        import std.stack;
        module app;
        def call( -- string) | -- string| { "x" } std.stack.call end
        def main( -- u8)
            0u8 1u8 std.math.gt
            | -- u8| { 2u8 }
            | -- u8| { 3u8 }
            std.cfg.if
        end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}

#[test]
fn structs() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        struct Outer(Inner, u8);
        struct Inner(string);
        struct Pair(u8, string);
        def nested(Outer -- string) Outer.0 Inner.0 end
        def roundtrip( -- u8, string) 1u8 "x" >Pair Pair> end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}

#[test]
fn modules() {
    TestExecutor::input((
        "app.by",
        r#"
        import deep.geometry;
        module app;
        struct Segment(deep.geometry.Point);
        def main( -- u8)
            1u8 2u8 >deep.geometry.Point >Segment
            Segment.0 deep.geometry.Point.1
        end
        "#,
    ))
    .add_file((
        "deep/geometry.by",
        "module deep.geometry; struct Point(u8, u8);",
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}

#[test]
fn symbols() {
    TestExecutor::input(("app.by", "module app; def main(Missing -- Missing) end"))
        .check()
        .assert_parse_ok()
        .assert_hir_err(|error| matches!(error, CompileError::SymbolNotFound { .. }));
}

#[test]
fn cycle() {
    TestExecutor::input(("app.by", "module app; struct A(B); struct B(A);"))
        .check()
        .assert_parse_ok()
        .assert_hir_err(|error| matches!(error, CompileError::RecursiveStruct { .. }));
}

#[test]
fn types() {
    let executor = TestExecutor::input((
        "app.by",
        r#"
        module app;
        def first( -- u8) end
        def second(u8 -- ) end
        def third( -- u8) "x" end
        "#,
    ))
    .check();
    let errors = executor.hir.unwrap().unwrap_err();

    assert_eq!(
        errors
            .iter()
            .filter(|error| matches!(error, CompileError::InvalidStack { .. }))
            .count(),
        3
    );
}
