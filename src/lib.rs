#![warn(clippy::all)]

// The build file results
// Build files are for the demos, so the wasm binaries can be included by default in the build
include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

mod app;
pub use app::TemplateApp;

mod rdx;
pub use rdx::RdxApp;

mod error;
pub use error::Error;

pub mod pest;

mod layer;
// mod plugins;
mod template;
mod utils;
