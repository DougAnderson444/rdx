//! Generated NewType wrappers around [html] crate so that we can
//! ensure the use of [Action] for actions, and [Handler] for functions.

use super::*;
use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

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

            #[doc = "Generate a new element with an action and handler."]
            #[doc = "This generates the equivalent of this in React:"]
            #[doc = "`onClick={handleClick}`"]
            #[doc = "Additonal method which takes an [Action] and [Handler]"]
            #[doc = "then calls applies them the the builder in a type safe way."]
            #[doc = concat!(
                "# Example\n",
                "```rust\n",
                "#![recursion_limit = \"512\"]\n",
                "\n",
                "use html_to_egui::{Action, Handler};\n",
                "use html_egui_bindgen::", stringify!($name), ";\n",
                "\n",
                "let mut tag = ", stringify!($name), "::new_with_func(\n",
                "    Action::OnClick,\n",
                "    Handler::builder()\n",
                "        .named(\"increment\".to_owned())\n",
                "        .args(vec![\"key\".to_owned()])\n",
                "        .build(),\n",
                ")\n",
                ".id(\"my-id\".to_owned())\n",
                ".class(\"my-class\".to_owned())\n",
                ".build();\n",
                "```\n"
            )]
            pub fn new_with_func(action: Action, func: Handler) -> Self {
                let mut div = Self::default();
                div.data(action, func);
                div
            }
        }
    };
}

/// A separate macro for those Builders that have a `text` method.
macro_rules! impl_hteg_text {
    ($name:ident, $foreign:ty, $builder:ty) => {
        #[doc = concat!(
            "The ", stringify!($name), " struct is a NewType wrapper around the [", stringify!($foreign), "] type.\n",
            "It returns a [", stringify!($builder), "] which is a builder for the [", stringify!($foreign), "] type."
        )]
        impl $name {
            /// Inserts inline rhai script into the html element.
            ///
            /// Conveneince method which internally wraps the rhai in `${}` block.
            /// This means that the rhai script must have escaped `{{` and `}}` blocks.
            /// ```ignore
            /// let inner_logic = r#"
            ///         if is_def_ver("my_rhai_variable")
            ///             {{ "Hello, World!" }}
            ///             else
            ///             {{ "Goodbye, World!" }} "#;
            /// let builder = $name::builder()
            ///     .with_rhai(inner_logic)
            ///     .build();
            ///
            /// // ..which interally uses:
            /// .text(r#"${ "#)
            /// .text(inner_logic)
            /// .text(r#" } "#)
            /// ```
            pub fn with_rhai(&mut self, rhai: impl Into<Cow<'static, str>>) -> &mut Self {
                self.0.text("${ ");
                self.0.text(rhai);
                self.0.text(" } ");
                self
            }
        }
    };
}

impl_hteg! { Division, html::text_content::Division, html::text_content::builders::DivisionBuilder}
impl_hteg! { Button, html::forms::Button, html::forms::builders::ButtonBuilder}
impl_hteg! { Paragraph, html::text_content::Paragraph, html::text_content::builders::ParagraphBuilder}
// html::forms::Label
impl_hteg! { Label, html::forms::Label, html::forms::builders::LabelBuilder}
// html::forms::Input
impl_hteg! { Input, html::forms::Input, html::forms::builders::InputBuilder}
// html::inline_text::Span
impl_hteg! { Span, html::inline_text::Span, html::inline_text::builders::SpanBuilder}

// Divisions with text
impl_hteg_text! { Division, html::text_content::Division, html::text_content::builders::DivisionBuilder}
// Paragraphs with text
impl_hteg_text! { Paragraph, html::text_content::Paragraph, html::text_content::builders::ParagraphBuilder}
// Labels with text
impl_hteg_text! { Label, html::forms::Label, html::forms::builders::LabelBuilder}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Action;
    use crate::Handler;

    #[test]
    fn test_division() {
        // This test tests:
        // 1) Regular builder
        // 2) Builder with action and handler
        // 3) Regular builder with with_rhai method
        // 4) Builder with action and handler with with_rhai method
        let mut _div = Division::builder()
            .id("my-id".to_owned())
            .class("my-class".to_owned())
            .build();

        let mut _div = Division::new_with_func(
            Action::OnClick,
            Handler::builder()
                .named("increment".to_owned())
                .args(vec!["key".to_owned()])
                .build(),
        )
        .id("my-id".to_owned())
        .class("my-class".to_owned())
        .build();

        let mut _div = Division::builder().with_rhai("Hello, World!").build();

        let mut _div: html::text_content::Division = Division::new_with_func(
            Action::OnClick,
            Handler::builder()
                .named("increment".to_owned())
                .args(vec!["key".to_owned()])
                .build(),
        )
        .with_rhai("Hello, World!")
        .id("an_id")
        .build();

        // with complex rhai script text that has inner Rust variables interpolated
        let hello = "Hello, Rust Variable!";
        let bye = "Goodbye, Rust Variable!";

        // uing format! + r#""# gives us the ability to interpolate Rust variables,
        // but we need to escape the `{}` with `{{}}` so that it doesn't get interpolated
        let inner_logic = format!(
            r#"
            if is_def_ver("my_rhai_variable")
                {{ `{hello}` }}
                else
                {{ `{bye}` }} "#
        );

        let div = Division::builder()
            .with_rhai(inner_logic)
            .build()
            .to_string();

        let parent_rhai = format!(r#"render(`{div}`)"#);

        assert_eq!(
            parent_rhai,
            r#"render(`<div>${ 
            if is_def_ver("my_rhai_variable")
                { `Hello, Rust Variable!` }
                else
                { `Goodbye, Rust Variable!` }  } </div>`)"#
        );
    }
}
