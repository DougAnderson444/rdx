#![recursion_limit = "512"] // for the html testing
#![warn(clippy::all)]
#![allow(static_mut_refs)] // dirs crate has warnings that break the CI build.

// The build file results, builtin components, are included in the build script
// Build files are for the demos, so the wasm binaries can be included by default in the build
include!(concat!(env!("OUT_DIR"), "/builtin_components.rs"));

mod app;
pub use app::TemplateApp;

mod rdx;
use rdx::RdxApp;
pub use rdx::{PluginDeets, State};

pub use wasm_component_layer;

mod error;
pub use error::Error;

/// HTML to egui (HTEG)
pub mod hteg;

mod futures;
pub mod layer;
pub use layer::LayerPlugin;

mod template;
pub mod utils;

/// A module to hold the build script
// not wasm32
#[cfg(not(target_arch = "wasm32"))]
mod build_script;
/// Export helper function to build the script
#[cfg(not(target_arch = "wasm32"))]
pub use build_script::build_script;
