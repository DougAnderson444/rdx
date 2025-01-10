use proc_macro::TokenStream;
use quote::quote;
use rhai::{Engine, ParseError};
use syn::{parse_macro_input, Expr, ExprLit, Lit, LitStr};
use syn::{Error, Result};

/// The `validate_rhai` macro takes a string literal as input,
/// compiles the Rhai script, and returns the input string
/// if the compilation is successful. If the compilation fails,
/// it generates a compile error with the error message.
///
/// # Example
/// ```rust
/// use validate_rhai_proc_macro::validate_rhai;
///
/// let script = validate_rhai!("let x = 1;"); // Returns "let x = 1;"
/// let invalid_script = validate_rhai!("let x = 1"); // Generates a compile error, missing semicolon
/// ```
#[proc_macro]
pub fn validate_rhai(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let script = input.value();

    let engine = Engine::new();
    match engine.compile(&script) {
        Ok(_) => TokenStream::from(quote! { #script }),
        Err(e) => {
            let ParseError(err_msg, position) = e;
            let line = position.line().unwrap_or(0);
            let column = position.position().unwrap_or(0);
            let error_msg = format!(
                "Rhai script compilation error at line {}, column {}: {}",
                line, column, err_msg
            );

            TokenStream::from(quote::quote_spanned! {
                proc_macro2::Span::call_site() =>
                compile_error!(#error_msg);
            })
        }
    }
}

#[proc_macro]
pub fn valid_rhai(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    // Try to evaluate the expression to a string at compile time
    let script = match expr_to_string(&expr) {
        Ok(s) => s,
        Err(e) => return error_tokens(&format!("Failed to evaluate expression: {}", e)),
    };

    // Validate the Rhai script
    let engine = Engine::new();
    match engine.compile(&script) {
        Ok(_) => quote! { #expr }.into(),
        Err(e) => error_tokens(&format!("Rhai compilation error: {}", e)),
    }
}

// Helper function t// Helper function to convert expression to string (if possible)
fn expr_to_string(expr: &Expr) -> Result<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => Ok(s.value()),
        // Handle potential macro expansion results
        Expr::Macro(expr_macro) => {
            // This is a simplification. In reality, you'd need to expand the macro
            // and recursively process its result, which is non-trivial.
            Err(Error::new_spanned(
                expr_macro.mac.path.clone(),
                "Macro expansion not supported",
            ))
        }
        // Handle other types of expressions as needed
        _ => Err(Error::new_spanned(
            expr,
            "Only string literals are supported in this context",
        )),
    }
}
// Helper function to generate error tokens
fn error_tokens(msg: &str) -> TokenStream {
    quote! {
        compile_error!(#msg);
    }
    .into()
}
