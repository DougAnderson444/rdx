# HTML to egui bindings

This small crate can be used to build html strings. It exists to enable type safety for the [html-to-egui](../html-to-egui) crate.

The reason it's in its own crate it because [html](https://docs.rs/html/latest/html/) crate can be hefty to include in wasm binaries. So this crate is used in the build step to create a static text file of rhai/RDX/html.
