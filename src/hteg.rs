//! HTML to egui (HTEG) converter.and renderer in egui.
use std::sync::{Arc, Mutex};

use scraper::{ElementRef, Html, Selector};
use wasm_component_layer::Value;

use crate::layer::{Inner, Instantiator};
use crate::template::{Template, TemplatePart};
use crate::Error;

/// Parses the html text into a Vec of [scraper::html::Select] elements.
/// Then renders the elements into egui UI components.
pub fn parse_and_render<T: Inner + Clone + Send + Sync>(
    ui: &mut egui::Ui,
    html: &str,
    plugin: Arc<Mutex<dyn Instantiator<T>>>,
) -> Result<(), Error> {
    let fragment = Html::parse_fragment(html);
    let top_selector = Selector::parse("html")?;
    for element in fragment.select(&top_selector) {
        render_element(ui, element, plugin.clone())?;
    }
    Ok(())
}

/// Wrapper struct to hold the [scraper::ElementRef] and the [HtmlElement] that
/// is being rendered into egui UI components.
struct ElementWrapper<'a> {
    html_element: HtmlElement,
    element_ref: ElementRef<'a>,
}

impl<'a> ElementWrapper<'a> {
    /// Creates a new [ElementWrapper] from the given [scraper::ElementRef].
    fn new(element_ref: ElementRef<'a>) -> Self {
        let html_element = HtmlElement::from_element(&element_ref);
        ElementWrapper {
            html_element,
            element_ref,
        }
    }

    /// Determines if this element matches the given selector.
    fn matches(&self, selector: &str) -> Result<bool, Error> {
        let selectors = Selector::parse(selector).map_err(|e| Error::Parse(e.to_string()))?;
        let m = selectors.matches(&self.element_ref);
        Ok(m)
    }

    /// Returns teh [HtmlElement]
    fn html_element(&self) -> &HtmlElement {
        &self.html_element
    }

    /// Returns a tuple of the functions and arguments for the given attribute.
    ///
    /// The attribute value is expected to be in the format `function_name(arg1, arg2, arg3)`.
    ///
    /// # Example
    /// ```ignore
    /// # use crate::hteg::ElementWrapper;
    /// # use scraper::{Html, Selector};
    /// let html = r#"<div data-on-click="increment(name)">...</div>"#;
    /// let fragment = Html::parse_fragment(html);
    /// let div_selector = Selector::parse("div").unwrap();
    /// let element = fragment.select(&div_selector).next().unwrap();
    /// let elw = ElementWrapper::new(element);
    /// let attr = "data-on-click";
    /// let (func, args) = elw.func_and_args(attr).unwrap();
    /// assert_eq!(func, "increment");
    /// assert_eq!(args, vec!["name"]);
    /// ```
    fn func_and_args(&self, attr: &str) -> Option<(&str, Vec<&str>)> {
        let attr = self.element_ref.value().attr(attr)?;
        let splits = attr.split('(').collect::<Vec<_>>();

        let func_name = splits[0];
        let func_args = splits[1]
            .trim_end_matches(')')
            .split(',')
            // mao on trim for whitespace, and filter on non-empty strings
            .filter_map(|v| {
                let v_trimmed = v.trim();
                if v_trimmed.is_empty() {
                    None
                } else {
                    Some(v_trimmed)
                }
            })
            .collect::<Vec<_>>();

        eprintln!("func_and_args: {:?} {:?}", func_name, func_args);
        Some((func_name, func_args))
    }
}

/// Enum to represent the HTML elements that can be rendered into egui UI components.
pub enum HtmlElement {
    /// Represents a div element. Divs are converted to [egui::Ui::vertical] by default.
    Div,
    /// Represents a button element. Buttons are converted to [egui::Button].
    Button,
    /// Represents an input element. Inputs are converted to [egui::TextEdit].
    Input,
    /// Represents a label element. Labels are converted to [egui::RichText].
    Label,
    /// Represents a span element. Spans are converted to [egui::RichText].
    Span,
    /// Paragraph element. Paragraphs are converted to [egui::RichText].
    Paragraph,
}

impl HtmlElement {
    /// Creates a new [HtmlElement] from the given [scraper::ElementRef].
    pub fn from_element(element: &ElementRef) -> Self {
        let tag_name = element.value().name();

        match tag_name {
            "div" => HtmlElement::Div,
            "button" => HtmlElement::Button,
            "input" => HtmlElement::Input,
            "label" => HtmlElement::Label,
            "span" => HtmlElement::Span,
            "p" => HtmlElement::Paragraph,
            _ => HtmlElement::Div,
        }
    }
}

/// Recurive function that walks the [scraper::ElementRef] and turns the
/// HTML into egui UI components.
pub fn render_element<T: Inner + Clone + Send + Sync>(
    ui: &mut egui::Ui,
    element: ElementRef,
    plugin: Arc<Mutex<dyn Instantiator<T>>>,
) -> Result<(), Error> {
    // fill the content template with scope values
    let content = {
        let content = element.text().collect::<String>();
        let template = Template::new(&content);
        let lock = plugin.lock().unwrap();
        let state = lock.store().data();
        let scope = state.clone().into_scope();
        // converts the rhai::Dynamic value to a string
        let entries = scope
            .iter()
            .map(|(k, _c, v)| (k, v.to_string()))
            .collect::<Vec<_>>();

        template.render(entries)
    };

    let elw = ElementWrapper::new(element);
    match elw.html_element() {
        HtmlElement::Div if elw.matches("div.flex-row")? => {
            ui.horizontal(|ui| {
                for child in element.child_elements() {
                    if let Err(e) = render_element(ui, child, plugin.clone()) {
                        tracing::error!("Error rendering child element: {:?}", e);
                    }
                }
            });
        }
        HtmlElement::Div => {
            ui.vertical(|ui| {
                for child in element.child_elements() {
                    if let Err(e) = render_element(ui, child, plugin.clone()) {
                        tracing::error!("Error rendering child element: {:?}", e);
                    }
                }
            });
        }
        HtmlElement::Button => {
            let color = match element.value().attr("color") {
                Some("green") => egui::Color32::from_rgb(100, 200, 100),
                Some("red") => egui::Color32::from_rgb(200, 100, 100),
                _ => ui.style().visuals.widgets.active.bg_fill,
            };

            let text = element.text().collect::<String>();
            if ui.add(egui::Button::new(&text).fill(color)).clicked() {
                if let Some((on_click, func_args)) = elw.func_and_args("data-on-click") {
                    let args = {
                        let mut lock = plugin.lock().unwrap();
                        let scope = lock.store_mut().data_mut().scope_mut();
                        func_args
                            .iter()
                            // ONLY use non-empty args, filter everything else out
                            // there can be zero arg ie) increment() where the return vec is zero
                            // length. That's ok.
                            .filter_map(|v| {
                                scope
                                    .get_value::<String>(v)
                                    .map(|val| Value::String(val.into()))
                            })
                            .collect::<Vec<_>>()
                    };

                    tracing::info!(
                        "Calling on_click function: {} with args: {:?} [length: {}]",
                        on_click,
                        args,
                        args.len()
                    );

                    let mut lock = plugin.lock().unwrap();
                    match lock.call(on_click, args.as_slice()) {
                        Ok(res) => {
                            tracing::info!("on_click response {:?}", res);
                        }
                        Err(e) => {
                            tracing::error!("on_click Error {:?}", e);
                        }
                    }
                }
            }
            ui.add_space(4.0);
        }
        HtmlElement::Input => {
            let is_password = element.value().attr("password") == Some("true");

            // get the first TemplatPart::Dynamic from template.parts.iter()
            // otherwisse return early
            // since input doesn't have a closing tag, we need to take the template from elsewhere
            // <input value="{{name}}" data-on-change="handle_change(name)">
            let template = Template::new(element.value().attr("value").unwrap_or_default());
            let Some(TemplatePart::Dynamic(var_name)) = template.parts.first() else {
                // nowhere to save the input, returning early
                return Err(Error::Parse("No variable name found".to_string()));
            };

            // Get the value of the variable from the rhai::Scope
            // Put the value into rhai::Scope as the value of the variable
            // Can I just linkt he rhai scope variable to the TextEdit widget?
            let mut lock = plugin.lock().unwrap();
            let mut scope = lock.store_mut().data_mut().scope_mut();

            if let Some(mut val) = scope.get_value::<String>(var_name.as_str()) {
                let single_line = egui::TextEdit::singleline(&mut val)
                    .desired_width(f32::INFINITY)
                    .password(is_password);
                let response = ui.add(single_line);
                if response.changed() {
                    // update the scope variable
                    scope.set_value(var_name.as_str(), val.clone());

                    // also call the on_change function if it exists
                    if let Some((on_change, func_args)) = elw.func_and_args("data-on-change") {
                        // if on_change is not empty, call the function
                        if !on_change.is_empty() {
                            let args = func_args
                                .iter()
                                .map(|v| {
                                    Value::String(
                                        scope.get_value::<String>(v).unwrap_or_default().into(),
                                    )
                                })
                                .collect::<Vec<_>>();

                            drop(scope);
                            drop(lock);

                            let mut lock = plugin.lock().unwrap();

                            if let Ok(value) = lock.call(on_change, args.as_slice()) {
                                match value {
                                    Some(Value::String(_s)) => {
                                        // TODO: act on return value(s)?
                                    }
                                    Some(Value::Bool(_)) => {}
                                    _ => {}
                                }
                            } else {
                                tracing::error!("Failed to call on_change function: {}", on_change);
                            }
                        }
                    }
                }
            } else {
                scope.set_value(var_name.as_str(), var_name.to_string());
            }
        }
        HtmlElement::Label | HtmlElement::Span | HtmlElement::Paragraph => {
            let size = match element.value().attr("size") {
                Some("small") => 14.0,
                Some("large") => 18.0,
                _ => 16.0,
            };

            ui.label(egui::RichText::new(content).size(size));
            ui.add_space(4.0);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_render() {
        let html = r#"
            <div>
                <button data-on-click="increment()">Increment</button>
                <button data-on-click="decrement()">Decrement</button>
                <p>Click to Start counting!</p>
            </div>
        "#;

        let fragment = Html::parse_fragment(html);
        let top_selector = Selector::parse("*").unwrap();
        for element in fragment.select(&top_selector) {
            tracing::info!("Rendering element {:?}", element);

            let tag_name = element.value().name();

            match tag_name {
                "div" => {
                    eprintln!("<div>");
                }
                "button" => {
                    eprintln!("\n<button>");
                }
                "p" => {
                    eprintln!("\n<p>");
                }
                _ => {}
            }
        }
    }

    // test func_and_args
    #[test]
    fn test_func_and_args() {
        let html = r#"
            <div>
                <button id=button1 data-on-click="increment(key)">Increment</button>
                <button id=button2 data-on-click="decrement(key, value)">Decrement</button>
                <button id=button3 data-on-click="reset()">Reset</button>
                <p>Click to Start counting!</p>
            </div>
        "#;

        let fragment = Html::parse_fragment(html);
        let button_1p_selector = Selector::parse("button#button1").unwrap();

        for element in fragment.select(&button_1p_selector) {
            let elw = ElementWrapper::new(element);
            let (func, args) = elw.func_and_args("data-on-click").unwrap();
            assert_eq!(func, "increment");
            assert_eq!(args, vec!["key"]);
        }

        let button_2p_selector = Selector::parse("button#button2").unwrap();

        for element in fragment.select(&button_2p_selector) {
            let elw = ElementWrapper::new(element);
            let (func, args) = elw.func_and_args("data-on-click").unwrap();
            assert_eq!(func, "decrement");
            assert_eq!(args, vec!["key", "value"]);
        }

        let button_3p_selector = Selector::parse("button#button3").unwrap();

        for element in fragment.select(&button_3p_selector) {
            let elw = ElementWrapper::new(element);
            let (func, args) = elw.func_and_args("data-on-click").unwrap();
            assert_eq!(func, "reset");
            assert!(args.is_empty());
        }
    }
}
