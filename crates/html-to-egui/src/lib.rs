//! Provides a NewType wrapper around [html::text_content::Division] to enforce
//! the use of [Action] for actions, and [Handler] for functions.
//!
//! All other [html] methods ar passed through.
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
