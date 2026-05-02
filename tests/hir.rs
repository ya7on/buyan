use buyan::error::CompileError;

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
    .assert_hir_ok();
}

#[test]
fn test_undefined_type() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main(A -- A) end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::SymbolNotFound { .. }));
}

#[test]
fn test_undefined_stackvar() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main(...A -- ...A) end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::SymbolNotFound { .. }));
}

#[test]
fn test_typevars_duplicate() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main<A, A>(A, A -- A, A) end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::SymbolAlreadyExists { .. }));
}

#[test]
fn test_stackvars_duplicate() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main<...A, ...A>(...A -- ...A) end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::SymbolAlreadyExists { .. }));
}

#[test]
fn test_stackvar_typevar_same_name() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main<...A, A>(...A -- ...A) end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::SymbolAlreadyExists { .. }));
}

#[test]
fn test_stackvar_used_as_typevar() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main<...A>(A -- A) end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::SymbolNotFound { .. }));
}

#[test]
fn test_typevar_used_as_stackvar() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main<A>(...A -- ...A) end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::SymbolNotFound { .. }));
}

#[test]
fn test_2_plus_2() {
    TestExecutor::input((
        "app.by",
        r#"
        import std.math;
        module app;
        def main( -- u8) 2u8 2u8 std.math.add end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}

#[test]
fn test_string() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main( -- string) "Hello, World!" end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}

#[test]
fn test_invalid_stack_out_type() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main( -- u8) "Hello, World!" end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::InvalidStack { .. }));
}

#[test]
fn test_lambda() {
    TestExecutor::input((
        "app.by",
        r#"
        import std.stack;
        module app;
        def foo( | u8 -- | -- ) std.stack.drop end
        def main( -- ) | u8 -- | { 67u8 } foo end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}

#[test]
fn test_invalid_lambda() {
    TestExecutor::input((
        "app.by",
        r#"
        import std.stack;
        module app;
        def foo( | string -- | -- ) std.stack.drop end
        def main( -- ) | u8 -- | { 67u8 } foo end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::InvalidStack { .. }));
}

#[test]
fn test_empty_body_typecheck() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main( -- ) 2u8 end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::InvalidStack { .. }));
}

#[test]
fn test_call() {
    TestExecutor::input((
        "app.by",
        r#"
        import std.stack;
        module app;
        def main( -- u8) | -- u8| { 67u8 } std.stack.call end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}

#[test]
fn test_lambda_check_exact_stack_in() {
    TestExecutor::input((
        "app.by",
        r#"
        import std.stack;
        module app;
        def takes_lambda(|string -- u8| --) std.stack.drop end
        def test( -- )
            |string, string -- u8| { std.stack.drop std.stack.drop 67u8 }
            takes_lambda
        end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::InvalidStack { .. }));
}

#[test]
fn test_lambda_check_exact_stack_out() {
    TestExecutor::input((
        "app.by",
        r#"
        import std.stack;
        module app;
        def takes_lambda(|string -- u8| --) std.stack.drop end
        def test( -- )
            |string -- u8, u8| { std.stack.drop 67u8 69u8 }
            takes_lambda
        end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::InvalidStack { .. }));
}

#[test]
fn test_if() {
    TestExecutor::input((
        "app.by",
        r#"
        import std.cfg;
        import std.math;
        module app;
        def main( -- u8)
            0u8 1u8 std.math.gt
            | -- u8| { 67u8 }
            | -- u8| { 69u8 }
            std.cfg.if
        end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}
