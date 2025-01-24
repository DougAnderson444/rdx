#![allow(dead_code)]

use std::{cell::RefCell, ops::Deref};

use ahash::HashSet;
use html_to_egui::{Action, Attribute, Selectors};
use markup5ever_rcdom::{Handle, NodeData};

use crate::{
    template::{Template, TemplatePart},
    Error,
};

/// Event Handler including the [Action] type and the function and arguments [FuncAndArgs]
#[derive(Debug, Clone)]
struct EvtHandler {
    ty: Action,
    details: FuncAndArgs,
}

impl EvtHandler {
    fn new(ty: Action, func_and_args: FuncAndArgs) -> Self {
        Self {
            ty,
            details: func_and_args,
        }
    }

    /// From an [Action] value
    fn new_from(ty: Action, val: &str) -> Result<Self, Error> {
        // trim end ')' ')' and split on '('
        let val = val.trim_end_matches(')');
        let mut parts = val.split('(');
        let Some(function) = parts.next() else {
            return Err(Error::Parse(format!(
                "No {} function found in : {}",
                ty, val
            )));
        };
        let args = parts
            .next()
            // only keep existant, filter out empty / non existant args
            // and trim any whitespace
            .map(|args| {
                args.split(',')
                    .filter(|arg| !arg.is_empty())
                    .map(|arg| arg.trim().to_string())
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self::new(
            Action::OnClick,
            FuncAndArgs {
                function: function.to_string(),
                args,
            },
        ))
    }

    /// Gets the function and args associated with an Action ty.
    fn func_and_args(&self) -> FuncAndArgs {
        FuncAndArgs {
            function: self.details.function.clone(),
            args: self.details.args.clone(),
        }
    }
}

/// Enum to represent the HTML elements that can be rendered into egui UI components.
#[derive(Debug, Clone)]
pub enum HtmlElement {
    /// Represents the root element of the HTML document.
    Html {
        /// The [HtmlElement] children of the html element.
        children: Vec<HtmlElement>,
    },
    /// Represents a div element. Divs are converted to [egui::Ui::vertical] by default.
    Div {
        /// The identlifier of the div element.
        id: Option<String>,
        /// The [HtmlElement] children of the div element.
        children: Vec<HtmlElement>,
        /// The text of this Div
        template: Template,
        /// The style of the div element.
        classes: HashSet<Selectors>,
    },
    /// Represents a button element. Buttons are converted to [egui::Button].
    Button(Button),
    /// Represents an input element. Inputs are converted to [egui::TextEdit].
    Input(Input),
    /// Represents a label element. Labels are converted to [egui::RichText].
    Label {
        /// The text of the label, expressed as a [Template].
        template: Template,
    },
    /// Represents a span element. Spans are converted to [egui::RichText].
    Span {
        /// The text of the span, expressed as a [Template].
        template: Template,
    },
    /// Paragraph element. Paragraphs are converted to [egui::RichText].
    Paragraph {
        /// The text of the paragraph, expressed as a [Template].
        template: Template,
    },
    /// The inner text of an element.
    Text {
        /// The text content of the element.
        contents: Template,
    },
    TextArea {
        /// The text content of the element.
        placeholder: Template,
    },
}

/// Button varaint details
#[derive(Debug, Clone)]
pub struct Button {
    /// The [Action] function, its type, and associated function arguments.
    evt_handlers: Vec<EvtHandler>,
    /// The text of the button, expressed as a [Template].
    template: Template,
    /// Button style
    style: Style,
}

/// Sturct to hold func and arg types
#[derive(Debug, Clone)]
pub struct FuncAndArgs {
    /// The function name
    pub function: String,
    /// The function arguments
    pub args: Vec<String>,
}

impl Button {
    /// Gets the function and args associated withthe given Action ty.
    pub fn func_and_args(&self, ty: Action) -> Option<FuncAndArgs> {
        self.evt_handlers.iter().find_map(|evt_handler| {
            if evt_handler.ty == ty {
                Some(evt_handler.details.clone())
            } else {
                None
            }
        })
    }
}

impl Button {
    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn template(&self) -> &Template {
        &self.template
    }
}

/// Input variant details
#[derive(Debug, Clone)]
pub struct Input {
    /// Whether the input is a password field or not.
    is_password: bool,
    /// The contents of the Input, expressed as a [Template].
    value: Template,
    /// The [Action] function, its type, and associated function arguments.
    evt_handlers: Vec<EvtHandler>,
    /// The variable name of the [rhaii:Scope] the imput is bound to
    var_name: String,
}

impl Input {
    /// Gets the function and args associated with the given Action ty.
    pub fn func_and_args(&self, ty: Action) -> Option<FuncAndArgs> {
        self.evt_handlers.iter().find_map(|evt_handler| {
            if evt_handler.ty == ty {
                Some(evt_handler.details.clone())
            } else {
                None
            }
        })
    }

    pub fn is_password(&self) -> bool {
        self.is_password
    }

    pub fn value(&self) -> &Template {
        &self.value
    }

    pub fn var_name(&self) -> &str {
        &self.var_name
    }

    /// The input element's template value as [Template]
    pub fn template(&self) -> &Template {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub struct Style {
    /// Color
    color: Option<Color>,
}

impl Style {
    fn new(color: Option<Color>) -> Self {
        Self { color }
    }

    pub fn color(&self) -> Option<&Color> {
        self.color.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct Color {
    /// The color of the button
    color: egui::Color32,
    /// The color of the button when hovered
    hovered: egui::Color32,
    /// The color of the button when clicked
    clicked: egui::Color32,
}

impl HtmlElement {
    // Define constants for the tag names
    const HTML: &'static str = "html";
    const DIV: &'static str = "div";
    const BUTTON: &'static str = "button";
    const INPUT: &'static str = "input";
    const LABEL: &'static str = "label";
    const SPAN: &'static str = "span";
    const PARAGRAPH: &'static str = "p";
    const TEXTAREA: &'static str = "textarea";

    // Method to get the string representation
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            HtmlElement::Html { .. } => Self::HTML,
            HtmlElement::Div { .. } => Self::DIV,
            HtmlElement::Button { .. } => Self::BUTTON,
            HtmlElement::Input { .. } => Self::INPUT,
            HtmlElement::Label { .. } => Self::LABEL,
            HtmlElement::Span { .. } => Self::SPAN,
            HtmlElement::Paragraph { .. } => Self::PARAGRAPH,
            HtmlElement::TextArea { .. } => Self::TEXTAREA,
            // No need
            HtmlElement::Text { .. } => unreachable!(),
        }
    }

    /// Recursive From Handle to [HtmlElement]
    pub(crate) fn from_node(node: &Handle) -> Option<Self> {
        match &node.data {
            NodeData::Document => {
                // Document node
                let children = node
                    .children
                    .borrow()
                    .iter()
                    .filter_map(HtmlElement::from_node)
                    .collect();
                Some(HtmlElement::Html { children })
            }
            NodeData::Element { name, attrs, .. } => {
                // Element node
                let tag = name.local.to_string();
                let id = attrs
                    .borrow()
                    .iter()
                    .find(|attr| *attr.name.local == *"id")
                    .map(|attr| attr.value.to_string());

                let children: Vec<HtmlElement> = node
                    .children
                    .borrow()
                    .iter()
                    .filter_map(HtmlElement::from_node)
                    .collect();

                let text = children
                    .iter()
                    .filter_map(|child| {
                        if let HtmlElement::Text { contents } = child {
                            Some(contents.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .concat();

                match tag.as_str() {
                    Self::HTML => Some(HtmlElement::Html { children }),
                    Self::DIV => {
                        let text = children
                            .iter()
                            .filter_map(|child| {
                                if let HtmlElement::Text { contents } = child {
                                    Some(contents.to_string())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" ");

                        // what we really want to do in order to make this
                        // matching type safe and extensible, is:
                        // 1. Get the class attribute
                        // 2. Split the value on whitespace to get each class name
                        // 3. Match each class name to a DivSelectors variant, if it exists (each
                        //    DivSelectors maps to a &str, which is available .as_str(), AsRef<str> or even
                        //    Deref)
                        // 4. If it exists, set it to the style
                        let mut classes = HashSet::default();
                        attrs.borrow().iter().for_each(|attr| {
                            if *attr.name.local == *"class" {
                                let s: &str = &attr.value;
                                if let Ok(selector) = Selectors::try_from(s) {
                                    classes.insert(selector);
                                }
                            }
                        });

                        Some(HtmlElement::Div {
                            id,
                            children,
                            template: Template::new(&text),
                            classes,
                        })
                    }
                    Self::BUTTON => {
                        let text = children
                            .iter()
                            .filter_map(|child| {
                                if let HtmlElement::Text { contents } = child {
                                    Some(contents.to_string())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .concat();
                        let evt_handlers = parse_evt_handlers(attrs);
                        Some(HtmlElement::Button(Button {
                            evt_handlers,
                            template: Template::new(&text),
                            style: Style::new(None),
                        }))
                    }
                    Self::INPUT => {
                        let is_password = attrs
                            .borrow()
                            .iter()
                            .any(|attr| *attr.name.local == *"password");

                        let value = attrs
                            .borrow()
                            .iter()
                            .find(|attr| *attr.name.local == *"value")
                            .map(|attr| attr.value.to_string())
                            .unwrap_or_default();

                        let template = Template::new(&value);
                        let template_clone = template.clone();
                        let Some(TemplatePart::Dynamic(var_name)) = template_clone.parts.first()
                        else {
                            // nowhere to save the input, returning early
                            return None;
                        };
                        let evt_handlers = parse_evt_handlers(attrs);

                        Some(HtmlElement::Input(Input {
                            is_password,
                            value: template,
                            evt_handlers,
                            var_name: var_name.to_string(),
                        }))
                    }
                    Self::LABEL => {
                        let text = Template::new(&text);
                        Some(HtmlElement::Label { template: text })
                    }
                    Self::SPAN => {
                        let text = Template::new(&text);
                        Some(HtmlElement::Span { template: text })
                    }
                    Self::PARAGRAPH => {
                        let text = Template::new(&text);
                        Some(HtmlElement::Paragraph { template: text })
                    }
                    Self::TEXTAREA => {
                        let placeholder = attrs
                            .borrow()
                            .iter()
                            .find(|attr| *attr.name.local == *"placeholder")
                            .map(|attr| attr.value.to_string())
                            .unwrap_or_default();
                        Some(HtmlElement::TextArea {
                            placeholder: Template::new(&placeholder),
                        })
                    }
                    _ => None,
                }
            }
            NodeData::Text { contents } => {
                // skip any contents that are only comprised of \n, whitespace, and no text
                if contents.borrow().trim().is_empty() {
                    return None;
                }
                // Text node
                Some(HtmlElement::Text {
                    contents: Template::new(&contents.borrow()),
                })
            }
            _ => None,
        }
    }

    /// The child elements
    pub(crate) fn child_elements(&self) -> Option<&Vec<HtmlElement>> {
        match self {
            HtmlElement::Html { children, .. } => Some(children),
            HtmlElement::Div { children, .. } => Some(children),
            _ => None,
        }
    }

    /// Adds a child to the element.
    pub(crate) fn add_child(&mut self, child: HtmlElement) {
        match self {
            HtmlElement::Html { children, .. } => children.push(child),
            HtmlElement::Div { children, .. } => children.push(child),
            _ => {}
        }
    }

    /// Returns the [Template] of the element, if any
    pub(crate) fn template(&self) -> &Template {
        match self {
            HtmlElement::Html { .. } => unreachable!(),
            HtmlElement::Label { template } => template,
            HtmlElement::Span { template } => template,
            HtmlElement::Paragraph { template } => template,
            HtmlElement::Div { template, .. } => template,
            HtmlElement::Button(Button { template, .. }) => template,
            HtmlElement::Input(Input { value, .. }) => value,
            HtmlElement::Text { contents } => contents,
            HtmlElement::TextArea { placeholder } => placeholder,
        }
    }
}

impl TryFrom<Handle> for HtmlElement {
    type Error = Error;

    fn try_from(value: Handle) -> Result<Self, Self::Error> {
        HtmlElement::from_node(&value).ok_or(Error::Parse("Unknown tag name".to_string()))
    }
}

impl From<HtmlElement> for &'static str {
    fn from(val: HtmlElement) -> Self {
        val.as_str()
    }
}

impl From<HtmlElement> for String {
    fn from(val: HtmlElement) -> Self {
        val.as_str().to_string()
    }
}

// impl into std::borrow::Cow<'static, str>>
impl From<HtmlElement> for std::borrow::Cow<'static, str> {
    fn from(val: HtmlElement) -> Self {
        val.as_str().into()
    }
}

impl Deref for HtmlElement {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

// parse event handlers
fn parse_evt_handlers(attrs: &RefCell<Vec<markup5ever::interface::Attribute>>) -> Vec<EvtHandler> {
    let mut evt_handlers = Vec::new();
    for attr in attrs.borrow().iter() {
        if let Ok(attribute) = Attribute::try_from(attr.name.local.to_string().as_str()) {
            match EvtHandler::new_from(attribute.into(), &attr.value) {
                Ok(evt_handler) => evt_handlers.push(evt_handler),
                Err(e) => {
                    tracing::error!("Invalid data-on-click function: {}", e);
                }
            }
        }
    }
    evt_handlers
}

#[cfg(test)]
mod tests {
    use super::*;

    // test EvtHandler
    #[test]
    fn test_evt_handler() {
        // test EvtHandler::new_from
        let evt_handler =
            EvtHandler::new_from(Action::OnClick, "function_name(arg1, arg2)").unwrap();

        assert_eq!(evt_handler.ty, Action::OnClick);
        assert_eq!(evt_handler.details.function, "function_name");
        assert_eq!(evt_handler.details.args, vec!["arg1", "arg2"]);
    }
}
