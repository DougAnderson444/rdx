#![feature(async_closure)]
#![warn(clippy::all)]

// The build file results
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

/// The Custom type module for exports
mod custom_types;
pub use custom_types::SharedUi;
