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

#[test]
fn test_struct_roundtrip() {
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
    .assert_hir_ok();
}

#[test]
fn test_nested_struct_with_forward_reference() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        struct Outer(Inner, u8);
        struct Inner(string);
        def main( -- Outer)
            "value" >Inner 7u8 >Outer
        end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}

#[test]
fn test_qualified_struct() {
    TestExecutor::input((
        "app.by",
        r#"
        import geometry;
        module app;
        def main( -- geometry.Point)
            2u8 3u8 >geometry.Point
        end
        "#,
    ))
    .add_file((
        "geometry.by",
        r#"
        module geometry;
        struct Point(u8, u8);
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}

#[test]
fn test_struct_dependency_from_imported_module() {
    TestExecutor::input((
        "app.by",
        r#"
        import geometry;
        module app;
        struct Segment(geometry.Point, geometry.Point);
        def main( -- Segment)
            1u8 2u8 >geometry.Point
            3u8 4u8 >geometry.Point
            >Segment
        end
        "#,
    ))
    .add_file((
        "geometry.by",
        r#"
        module geometry;
        struct Point(u8, u8);
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_ok();
}

#[test]
fn test_struct_pack_type_mismatch() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        struct Pair(u8, string);
        def main( -- Pair)
            "wrong" 7u8 >Pair
        end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::InvalidStack { .. }));
}

#[test]
fn test_unpack_wrong_struct() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        struct A(u8);
        struct B(u8);
        def main( -- u8)
            7u8 >A B>
        end
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::InvalidStack { .. }));
}

#[test]
fn test_unknown_struct_field() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        struct Wrapper(Missing);
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::SymbolNotFound { .. }));
}

#[test]
fn test_recursive_struct() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        struct A(B);
        struct B(A);
        "#,
    ))
    .check()
    .assert_parse_ok()
    .assert_hir_err(|err| matches!(err, CompileError::RecursiveStruct { .. }));
}
