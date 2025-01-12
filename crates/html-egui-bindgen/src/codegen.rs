//! Generated NewType wrappers around [html] crate so that we can
//! ensure the use of [Action] for actions, and [Handler] for functions.

use super::*;
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
        #[doc = concat!(
            "The ", stringify!($name), " struct is a NewType wrapper around the [", stringify!($foreign), "] type.\n",
            "It returns a [", stringify!($builder), "] which is a builder for the [", stringify!($foreign), "] type."
        )]
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
            #[doc = concat!(
                "Builder method to create a new instance of the ", stringify!($name), " struct.\n",
                "It's a pass through to the default method."
            )]
            pub fn builder() -> Self {
                Self::default()
            }

            /// Additonal method which takes an [Action] and [Handler]
            /// then calls applies them the the builder in a type safe way.
            #[doc = concat!("
            # Example
            ```rust
            #![recursion_limit = \"512\"]

            use html_to_egui::{Action, Handler};
            use html_egui_bindgen::", stringify!($name), ";

            let mut tag = ", stringify!($name), "::new_with_func(
                Action::OnClick,
                Handler::builder()
                    .named(\"increment\".to_owned())
                    .args(vec![\"key\".to_owned()])
                    .build(),
            )
            .id(\"my-id\".to_owned())
            .class(\"my-class\".to_owned())
            .build();
            ```
            ")]
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
