#![warn(clippy::all)]

mod app;
pub use app::TemplateApp;

mod rdx;
pub use rdx::RdxApp;

mod error;
pub use error::Error;

mod custom_syntax;
mod dsl;
pub mod pest;
