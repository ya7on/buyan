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
