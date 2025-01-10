//! Provides a NewType wrapper around [html::text_content::Division] to enforce
//! the use of [crate::hteg::Action] for actions, and [crate::hteg::Handler] for functions.
//!
//! All other [html] methods ar passed through.
#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use html::text_content;
use std::ops::{Deref, DerefMut};

pub struct Division(text_content::builders::DivisionBuilder);

impl Deref for Division {
    type Target = text_content::builders::DivisionBuilder;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Division {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Division {
    fn default() -> Self {
        Self(html::text_content::Division::builder())
    }
}

impl Division {
    /// Additonal method which takes an [crate::hteg::Action] and [crate::hteg::Handler]
    /// then calls applies them the the builder in a type safe way.
    ///
    /// # Example
    /// ```rust
    /// #![recursion_limit = "512"]
    ///
    /// use rdx::hteg::Action;
    /// use rdx::hteg::Handler;
    /// use rdx::hteg::Division;
    ///
    /// let mut div = Division::new_with_func(Action::OnClick,
    ///                        Handler::builder()
    ///                            .named("increment".to_owned())
    ///                            .args(vec!["key".to_owned()])
    ///                            .build());
    /// div.build();
    /// ```
    pub fn new_with_func(action: crate::hteg::Action, func: crate::hteg::Handler) -> Self {
        let mut div = Self::default();
        div.data(action, func);
        div
    }
}

/// Button NewType wrapper around [html::text_content::Button]
/// to enforce the use of [crate::hteg::Action] for actions, and [crate::hteg::Handler] for functions.
pub struct Button(html::forms::builders::ButtonBuilder);

impl Default for Button {
    fn default() -> Self {
        Self(html::forms::Button::builder())
    }
}

impl Deref for Button {
    type Target = html::forms::builders::ButtonBuilder;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Button {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Button {
    /// Additonal construction method which takes an [crate::hteg::Action] and [crate::hteg::Handler]
    /// then calls applies them the the builder in a type safe way.
    ///
    /// # Example
    /// ```rust
    /// #![recursion_limit = "512"]
    ///
    /// use rdx::hteg::{Button, Action, Handler};
    ///
    /// let mut button = Button::new_with_func(Action::OnClick,
    ///                       Handler::builder()
    ///                       .named("increment".to_owned())
    ///                       .args(vec!["key".to_owned()])
    ///                       .build());
    /// button.build();
    /// ```
    pub fn new_with_func(action: crate::hteg::Action, func: crate::hteg::Handler) -> Self {
        let mut button = Self::default();
        button.data(action, func);
        button
    }
}
