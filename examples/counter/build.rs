//! Build script to build the RDX template at compile time.
//! RDX is just Rhai and Html text, so this makes sense to do so.
//! This saves the runtime from size and compute.
//! Also, it enables us to check the Rhai to ensure it compiles,
//! at Rust compile time.
//!
//! This build script uses the html crate to programmatically build
//! the html portion of RDX with type safety. This means no typos!
//!
//! Then this html is injected into the Rhai by using
//!
//! ```ignore
//! let html = "<p>some html text</p>"
//! let rhai = format!(r#" the {html} "#);
//! ````
#![recursion_limit = "512"]

use std::env;
use std::fs;
use std::path::PathBuf;

use html_egui_bindgen::{Button, Division, Paragraph};
use html_to_egui::{Action, DivSelectors, Handler};
use rhai::Engine;
use rhai::ParseError;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap_or_default();
    let dest_path = PathBuf::from(out_dir).join("counter.rhai");

    let rhai_verified = assert_rhai_compiles();

    fs::write(&dest_path, rhai_verified).expect("Failed to write to file");

    println!("cargo:rerun-if-changed=build.rs");
}

fn gen_script() -> String {
    let increment_button = Button::new_with_func(
        Action::OnClick,
        // the function name must match the wasm function name in this file
        // converted into kebab-case (so my_function becomes my-function)
        Handler::builder()
            .named("increment-count".to_string())
            .build(),
    )
    .text("Increment")
    .build();

    let decrement_button = Button::new_with_func(
        Action::OnClick,
        Handler::builder().named("decrement".to_string()).build(),
    )
    .text("Decrement")
    .build();

    let no_def_count_para = Paragraph::builder()
        .text("Click to Start counting!")
        .build();

    let def_count_para = Paragraph::builder().text("Count is: {{count}}").build();

    let def_count = Division::builder()
        .push(increment_button.clone())
        .push(decrement_button.clone())
        .push(def_count_para)
        .class(DivSelectors::FlexRow)
        .build()
        .to_string();

    let no_def_count = Division::builder()
        .push(increment_button)
        .push(decrement_button)
        .push(no_def_count_para)
        .build()
        .to_string();

    format!(
        r#"
            // A simple counter example.
            // This is a Rhai script that will be compiled at build time.
            // We use Rhai for logic control flow of which html to render.
            // We use the html to render the egui to the end user.

            // We check if the count is defined, if not we show the no_def_count html
            if !is_def_var("count") || count == "0" {{

                // render is registered with the rhai engine to render the html
                // in the html, any functions used will be wasm functions in the main source.
                render(`{no_def_count}`)

            }} else {{

                render(`{def_count}`)

            }}
        "#
    )
}

/// Compile the Rhai in advamce, give a Rustc compile error if it fails.
fn assert_rhai_compiles() -> String {
    let script = gen_script();

    let engine = Engine::new();
    match engine.compile(&script) {
        Ok(_) => (),
        Err(e) => {
            let ParseError(err_msg, position) = e;
            let line = position.line().unwrap_or(0);
            let column = position.position().unwrap_or(0);
            let error_msg = format!(
                "Rhai script compilation error at line {}, column {}: {}",
                line, column, err_msg
            );

            // If compilation fails, emit a compile-time error
            println!(
                "cargo:warning=Rhai script compilation error: {:?}",
                error_msg
            );
            panic!("Rhai script compilation failed");
        }
    }

    script
}