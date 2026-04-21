use crate::common::executor::TestExecutor;

mod common;

#[test]
fn test_simple_program() {
    TestExecutor::input((
        "app",
        r#"
        module app;
        def main( -- ) end
        "#,
    ))
    .check()
    .assert_lex_ok()
    .assert_parse_ok();
}

#[test]
fn test_imports() {
    TestExecutor::input((
        "app",
        r#"
        import foo;
        import bar;
        import really.long.name;

        module app;
        "#,
    ))
    .check()
    .assert_lex_ok()
    .assert_parse_ok();
}

#[test]
fn test_typevars() {
    TestExecutor::input((
        "app",
        r#"
        module app;
        def main<A, B, C>(A, B, C -- C, B, A) end
        "#,
    ))
    .check()
    .assert_lex_ok()
    .assert_parse_ok();
}

#[test]
fn test_stackvars() {
    TestExecutor::input((
        "app",
        r#"
        module app;
        def main<...A, ...B>(...A -- ...B) end
        "#,
    ))
    .check()
    .assert_lex_ok()
    .assert_parse_ok();
}

#[test]
fn test_stackvars_with_typevars() {
    TestExecutor::input((
        "app",
        r#"
        module app;
        def main<...S, A, B>(...S, A, B -- ...S, B, A) end
        "#,
    ))
    .check()
    .assert_lex_ok()
    .assert_parse_ok();
}

#[test]
fn test_intrinsics_attribute() {
    TestExecutor::input((
        "app",
        r#"
        module app;
        #[intrinsic]
        def main( -- ) end
        "#,
    ))
    .check()
    .assert_lex_ok()
    .assert_parse_ok();
}
