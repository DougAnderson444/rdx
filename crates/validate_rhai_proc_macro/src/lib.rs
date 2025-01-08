use proc_macro::TokenStream;
use quote::quote;
use rhai::{Engine, ParseError};
use syn::{parse_macro_input, LitStr};

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
