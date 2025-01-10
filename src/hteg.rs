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
}

/// Enum to represent the HTML elements that can be rendered into egui UI components.
pub enum HtmlElement {
    /// Represents a div element. Divs are converted to [egui::Ui::vertical] by default.
    Div(Vec<Attribute>),
    /// Represents a button element. Buttons are converted to [egui::Button].
    Button(Vec<Attribute>),
    /// Represents an input element. Inputs are converted to [egui::TextEdit].
    Input(Vec<Attribute>),
    /// Represents a label element. Labels are converted to [egui::RichText].
    Label(Vec<Attribute>),
    /// Represents a span element. Spans are converted to [egui::RichText].
    Span(Vec<Attribute>),
    /// Paragraph element. Paragraphs are converted to [egui::RichText].
    Paragraph(Vec<Attribute>),
}

pub struct Attribute {
    name: String,
    value: String,
}

impl HtmlElement {
    /// Creates a new [HtmlElement] from the given [scraper::ElementRef].
    pub fn from_element(element: &ElementRef) -> Self {
        let tag_name = element.value().name();
        let attributes = element
            .value()
            .attrs()
            .map(|(name, value)| Attribute {
                name: name.to_string(),
                value: value.to_string(),
            })
            .collect::<Vec<_>>();

        match tag_name {
            "div" => HtmlElement::Div(attributes),
            "button" => HtmlElement::Button(attributes),
            "input" => HtmlElement::Input(attributes),
            "label" => HtmlElement::Label(attributes),
            "span" => HtmlElement::Span(attributes),
            "p" => HtmlElement::Paragraph(attributes),
            _ => HtmlElement::Div(attributes),
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
    // helper closure to get the function and its arguments from the element's attribute
    let func_and_args = |attr: &str| -> Option<(&str, Vec<&str>)> {
        match element.value().attr(attr) {
            Some(attr) => {
                let func_and_args = attr.split('(').collect::<Vec<_>>();
                let on_click = func_and_args[0];
                let func_args = func_and_args[1]
                    .split(',')
                    .map(|v| v.trim())
                    .collect::<Vec<_>>();
                Some((on_click, func_args))
            }
            None => None,
        }
    };

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
        HtmlElement::Div(_attrs) if elw.matches("div.flex-row")? => {
            ui.horizontal(|ui| {
                for child in element.child_elements() {
                    if let Err(e) = render_element(ui, child, plugin.clone()) {
                        tracing::error!("Error rendering child element: {:?}", e);
                    }
                }
            });
        }
        HtmlElement::Div(_attrs) => {
            ui.vertical(|ui| {
                for child in element.child_elements() {
                    if let Err(e) = render_element(ui, child, plugin.clone()) {
                        tracing::error!("Error rendering child element: {:?}", e);
                    }
                }
            });
        }
        HtmlElement::Button(_attrs) => {
            let color = match element.value().attr("color") {
                Some("green") => egui::Color32::from_rgb(100, 200, 100),
                Some("red") => egui::Color32::from_rgb(200, 100, 100),
                _ => ui.style().visuals.widgets.active.bg_fill,
            };

            let text = element.text().collect::<String>();
            if ui.add(egui::Button::new(&text).fill(color)).clicked() {
                if let Some((on_click, func_args)) = func_and_args("data-on-click") {
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
        HtmlElement::Input(_attrs) => {
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
                    if let Some(on_change_val) = element.value().attr("data-on-change") {
                        // get the fn and its args, example <div data-on-change="handle_change(arg1, arg2)">...</div>
                        // it' a vec of strings ["handle_change", "arg1, arg2)"]
                        // split by '(' and ','
                        let func_and_args = on_change_val.split('(').collect::<Vec<_>>();
                        let on_change = func_and_args[0];
                        let func_args = func_and_args[1]
                            .split(',')
                            // trim any whitespace, front and/or end
                            .map(|v| v.trim())
                            .collect::<Vec<_>>();

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
        HtmlElement::Label(_attrs) | HtmlElement::Span(_attrs) | HtmlElement::Paragraph(_attrs) => {
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
}
