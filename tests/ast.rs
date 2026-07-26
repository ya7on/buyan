use buyan::error::CompileError;

use crate::common::executor::TestExecutor;

mod common;

#[test]
fn test_invalid_token() {
    TestExecutor::input((
        "app.by",
        r#"
        !
        "#,
    ))
    .check()
    .assert_parse_err(|err| matches!(err, CompileError::UnexpectedToken { .. }));
}

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
    .assert_parse_ok();
}

#[test]
fn test_imports() {
    TestExecutor::input((
        "app.by",
        r#"
        import foo;
        import bar;
        import really.long.name;

        module app;
        "#,
    ))
    .add_file(("foo.by", "module foo;"))
    .add_file(("bar.by", "module bar;"))
    .add_file(("really/long/name.by", "module really.long.name;"))
    .check()
    .assert_parse_ok();
}

#[test]
fn test_import_std() {
    TestExecutor::input((
        "app.by",
        r#"
        import std.stack;
        module app;
        def main( -- ) end
        "#,
    ))
    .check()
    .assert_parse_ok();
}

#[test]
fn test_import_err() {
    TestExecutor::input((
        "app.by",
        r#"
        import foo;
        module app;
        def main( -- ) end
        "#,
    ))
    .check()
    .assert_parse_err(|err| {
        matches!(
            err,
            CompileError::ImportError { path, .. } if path == "foo.by"
        )
    });
}

#[test]
fn test_typevars() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main<A, B, C>(A, B, C -- C, B, A) end
        "#,
    ))
    .check()
    .assert_parse_ok();
}

#[test]
fn test_stackvars() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main<...A, ...B>(...A -- ...B) end
        "#,
    ))
    .check()
    .assert_parse_ok();
}

#[test]
fn test_stackvars_with_typevars() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main<...S, A, B>(...S, A, B -- ...S, B, A) end
        "#,
    ))
    .check()
    .assert_parse_ok();
}

#[test]
fn test_intrinsics_attribute() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        #[intrinsic]
        def main( -- ) end
        "#,
    ))
    .check()
    .assert_parse_ok();
}

#[test]
fn test_struct_pack_unpack() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        struct Point(u8, u8);
        def main( -- u8, u8)
            2u8 3u8 >Point Point>
        end
        "#,
    ))
    .check()
    .assert_parse_ok();
}

#[test]
fn test_empty_and_qualified_struct_syntax() {
    TestExecutor::input((
        "app.by",
        r#"
        import geometry;
        module app;
        struct Unit();
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
    .assert_parse_ok();
}

#[test]
fn test_struct_after_word_is_invalid() {
    TestExecutor::input((
        "app.by",
        r#"
        module app;
        def main( -- ) end
        struct Point(u8, u8);
        "#,
    ))
    .check()
    .assert_parse_err(|err| matches!(err, CompileError::ParseError { .. }));
}
