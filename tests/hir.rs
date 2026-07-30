use buyan::error::{DiagnosticKind, DiagnosticMessage};

use crate::common::executor::TestExecutor;

mod common;

#[test]
fn values() {
    TestExecutor::input((
        "app.by",
        r#"
        import std.math;
        module app;
        def main( -- u8, [u8; 1]) 1u8 2u8 std.math.add "x" end
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
        def call( -- [u8; 1]) | -- [u8; 1]| { "x" } std.stack.call end
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
        struct Inner([u8; 1]);
        struct Pair(u8, [u8; 1]);
        def nested(Outer -- [u8; 1]) Outer.0 Inner.0 end
        def roundtrip( -- u8, [u8; 1]) 1u8 "x" >Pair Pair> end
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
        .assert_hir_err(|error| matches!(error, DiagnosticMessage::SymbolNotFound { .. }));
}

#[test]
fn cycle() {
    TestExecutor::input(("app.by", "module app; struct A(B); struct B(A);"))
        .check()
        .assert_parse_ok()
        .assert_hir_err(|error| matches!(error, DiagnosticMessage::RecursiveStruct { .. }));
}

#[test]
fn types() -> Result<(), String> {
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
    let hir = executor
        .hir
        .ok_or_else(|| "HIR stage did not run".to_string())?;
    let errors = hir
        .err()
        .ok_or_else(|| "HIR stage unexpectedly succeeded".to_string())?;

    assert_eq!(
        errors
            .iter()
            .filter(|diagnostic| {
                diagnostic.kind == DiagnosticKind::Error
                    && matches!(&diagnostic.message, DiagnosticMessage::InvalidStack { .. })
            })
            .count(),
        3
    );
    Ok(())
}
