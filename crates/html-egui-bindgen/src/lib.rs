#![doc = include_str!("../README.md")]
#![recursion_limit = "512"] // for the html testing

mod codegen;

pub use codegen::*;

pub use html_to_egui::{Action, Handler};

// Test the README.md code snippets
#[cfg(doctest)]
pub struct ReadmeDoctests;
