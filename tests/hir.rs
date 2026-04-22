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

// #[test]
// fn test_lambda() {
//     TestExecutor::input((
//         "app.by",
//         r#"
//         import std.stack;
//         module app;
//         def main( -- | -- u8|) | -- u8 | { 67u8 } end
//         "#,
//     ))
//     .check()
//     .assert_parse_ok()
//     .assert_hir_ok();
// }

// #[test]
// fn test_call() {
//     TestExecutor::input((
//         "app.by",
//         r#"
//         import std.stack;
//         module app;
//         def main( -- u8) | -- u8| { 67u8 } std.stack.call end
//         "#,
//     ))
//     .check()
//     .assert_parse_ok()
//     .assert_hir_ok();
// }

// #[test]
// fn test_empty_body_typecheck() {
//     TestExecutor::input((
//         "app.by",
//         r#"
//         module app;
//         def main( -- ) 2u8 end
//         "#,
//     ))
//     .check()
//     .assert_parse_ok()
//     .assert_hir_err(|err| matches!(err, CompileError::InvalidStack { .. }));
// }
