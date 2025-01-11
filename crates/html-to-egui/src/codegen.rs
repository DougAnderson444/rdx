//! Generated NewType wrappers around [html] crate so that we can
//! ensure the use of [Action] for actions, and [Handler] for functions.

use super::{Action, Handler};
use std::fmt::Display;
use std::ops::{Deref, DerefMut};

//pub struct Division(text_content::builders::DivisionBuilder);
//
//impl Deref for Division {
//    type Target = text_content::builders::DivisionBuilder;
//
//    fn deref(&self) -> &Self::Target {
//        &self.0
//    }
//}
//
//impl DerefMut for Division {
//    fn deref_mut(&mut self) -> &mut Self::Target {
//        &mut self.0
//    }
//}
//
//impl Default for Division {
//    fn default() -> Self {
//        Self(html::text_content::Division::builder())
//    }
//}
//
//impl Division {
//    pub fn builder() -> Self {
//        Self::default()
//    }
//
//    /// Additonal method which takes an [Action] and [Handler]
//    /// then calls applies them the the builder in a type safe way.
//    ///
//    /// # Example
//    /// ```rust
//    /// #![recursion_limit = "512"]
//    ///
//    /// use rdx::hteg::Action;
//    /// use rdx::hteg::Handler;
//    /// use rdx::hteg::Division;
//    ///
//    /// let mut div = Division::new_with_func(
//    ///     Action::OnClick,
//    ///     Handler::builder()
//    ///         .named("increment".to_owned())
//    ///         .args(vec!["key".to_owned()])
//    ///         .build(),
//    /// )
//    /// .build();
//    ///
//    /// assert_eq!(div.to_string(), "<div data-on-click=\"increment(key)\"></div>");
//    /// ```
//    pub fn new_with_func(action: Action, func: Handler) -> Self {
//        let mut div = Self::default();
//        div.data(action, func);
//        div
//    }
//}

// there will be a lot of duplication here, but it is necessary to enforce the type safety
// Let's make a macro to generate these for us:
macro_rules! impl_hteg {
    ($name:ident, $foreign:ty, $builder:ty) => {
        pub struct $name($builder);

        impl Default for $name {
            fn default() -> Self {
                Self(<$foreign>::builder())
            }
        }

        impl Deref for $name {
            type Target = $builder;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl $name {
            pub fn builder() -> Self {
                Self::default()
            }

            /// Additonal method which takes an [Action] and [Handler]
            /// then calls applies them the the builder in a type safe way.
            ///
            /// # Example
            /// ```rust
            /// #![recursion_limit = "512"]
            ///
            /// use rdx::hteg::{Action, Handler, $name};
            ///
            /// let mut tag = $name::new_with_func(
            ///         Action::OnClick,
            ///         Handler::builder()
            ///         .named("increment".to_owned())
            ///         .args(vec!["key".to_owned()])
            ///         .build(),
            ///    )
            ///    .build();
            ///
            ///    assert_eq!(tag.to_string(), "<tag data-on-click=\"increment(key)\"></tag>");
            ///    ```
            pub fn new_with_func(action: Action, func: Handler) -> Self {
                let mut div = Self::default();
                div.data(action, func);
                div
            }
        }
    };
}

impl_hteg! { Division, html::text_content::Division, html::text_content::builders::DivisionBuilder}

impl_hteg! { Button, html::forms::Button, html::forms::builders::ButtonBuilder}

impl_hteg! { Paragraph, html::text_content::Paragraph, html::text_content::builders::ParagraphBuilder}

///// Button NewType wrapper around [html::text_content::Button]
///// to enforce the use of [Action] for actions, and [Handler] for functions.
/////
///// All other [html] methods are passed through.
/////
///// # Example
///// ```rust
///// #![recursion_limit = "512"]
///// use rdx::hteg::Button;
///// use rdx::hteg::{Action, Handler};
/////
///// let button = Button::new_with_func(
/////     Action::OnClick,
/////     Handler::builder()
/////         .named("increment".to_owned())
/////         .args(vec!["key".to_owned()])
/////         .build(),
///// )
///// // now we can use [html::text_content::Button] methods to add more optional details
///// .id("button1")
///// .text("Increment")
///// .build();
/////
///// assert_eq!(button.to_string(), "<button id=\"button1\" data-on-click=\"increment(key)\">Increment</button>");
///// ```
//pub struct Button(html::forms::builders::ButtonBuilder);
//
//impl Default for Button {
//    fn default() -> Self {
//        Self(html::forms::Button::builder())
//    }
//}
//
//impl Deref for Button {
//    type Target = html::forms::builders::ButtonBuilder;
//
//    fn deref(&self) -> &Self::Target {
//        &self.0
//    }
//}
//
//impl DerefMut for Button {
//    fn deref_mut(&mut self) -> &mut Self::Target {
//        &mut self.0
//    }
//}
//
//impl Button {
//    /// Additonal construction method which takes an [Action] and [Handler]
//    /// then calls applies them the the builder in a type safe way.
//    ///
//    /// # Example
//    /// ```rust
//    /// #![recursion_limit = "512"]
//    ///
//    /// use rdx::hteg::{Button, Action, Handler};
//    ///
//    /// let mut button = Button::new_with_func(Action::OnClick,
//    ///                       Handler::builder()
//    ///                       .named("increment".to_owned())
//    ///                       .args(vec!["key".to_owned()])
//    ///                       .build());
//    /// button.build();
//    /// ```
//    pub fn new_with_func(action: Action, func: Handler) -> Self {
//        let mut button = Self::default();
//        button.data(action, func);
//        button
//    }
//}
