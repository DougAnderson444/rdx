//! Integration tests for the macro
use validate_rhai_proc_macro::*;

/// The `test_validate_rhai` test case checks that the macro
/// returns the input string when the compilation is successful.
#[test]
fn test_validate_rhai_parenthesis() {
    let output = validate_rhai! { "let x = 42;" };
    assert_eq!(output, "let x = 42;");
}

#[test]
fn test_validate_rhai_brackets() {
    let output = validate_rhai!("let x = 42;");
    assert_eq!(output, "let x = 42;");
}

// test failure
#[test]
fn test_validate_rhai_multiline() {
    let output = validate_rhai! {"
        let x = 42;
        let y = 69;
        "};
    assert_eq!(
        output,
        "\n        let x = 42;\n        let y = 69;\n        "
    );
}
