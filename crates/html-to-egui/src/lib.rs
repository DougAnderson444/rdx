//! Provides a NewType wrapper around [html] crate to enforce
//! the use of [Action] for actions, and [Handler] for functions.
//!
//! This gives us type safety when defining actions and functions,
//! making parsing the html to egui more robust.
//!
//! All other [html] methods ar passed through and can be called also.
//!
//! [html] NewType code is generated in the [codegen] module using a macro.
//!
//! Other modules include [selectors] for css selectors,
//! [attribute] for html attributes, and [action] for actions.
//! The [handler] module provides a builder for [Handler],
//! which is used to define functions for actions.

#![recursion_limit = "512"] // needed for html crate
#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod action;
pub use action::Action;

mod attribute;
pub use attribute::Attribute;

mod codegen;
pub use codegen::*;

mod handler;
pub use handler::Handler;

mod selectors;
pub use selectors::*;

use html::text_content;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
