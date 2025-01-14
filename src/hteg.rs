//! HTML to egui (HTEG) converter.and renderer in egui.
mod element_parser;
mod types;

use element_parser::Parser;
use html_to_egui::{Action, Selectors};
use types::{FuncAndArgs, HtmlElement};

use std::sync::{Arc, Mutex};

use wasm_component_layer::Value;

use crate::layer::{Inner, Instantiator};
use crate::template::TemplatePart;
use crate::Error;

/// Parses the html and renders to egui for us.
#[derive(Default, Clone)]
pub struct HtmlToEgui {
    parser: Parser,
}

impl HtmlToEgui {
    /// Parses the html text into a Vec of [scraper::html::Select] elements.
    /// Then renders the elements into egui UI components.
    pub fn parse_and_render<T: Inner + Clone + Send + Sync>(
        &mut self,
        ui: &mut egui::Ui,
        html: &str,
        plugin: Arc<Mutex<dyn Instantiator<T>>>,
    ) -> Result<(), Error> {
        let html_ast = self.parser.parse(html)?;
        render_element(ui, &html_ast, plugin.clone())?;
        Ok(())
    }
}

/// Recurive function that walks the [scraper::ElementRef] and turns the
/// HTML into egui UI components.
fn render_element<T: Inner + Clone + Send + Sync>(
    ui: &mut egui::Ui,
    element: &HtmlElement,
    plugin: Arc<Mutex<dyn Instantiator<T>>>,
) -> Result<(), Error> {
    // get the content scope values
    // converts the rhai::Dynamic value to a string
    let entries = {
        let lock = plugin.lock().unwrap();
        let state = lock.store().data();
        let scope = state.clone().into_scope();
        scope
            .iter()
            .map(|(k, _c, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<_>>()
    };

    match element {
        HtmlElement::Html { children, .. } => {
            for child in children {
                render_element(ui, child, plugin.clone())?;
            }
        }
        HtmlElement::Div {
            template: _, style, ..
        } => {
            let inner = |ui: &mut egui::Ui| {
                if element.child_elements().is_some() {
                    for child in element.child_elements().unwrap() {
                        if let Err(e) = render_element(ui, child, plugin.clone()) {
                            tracing::error!("Error rendering child element: {:?}", e);
                        }
                    }
                }
            };
            match style {
                Selectors::FlexRow => ui.horizontal(|ui| {
                    inner(ui);
                }),
                Selectors::None => ui.vertical(|ui| {
                    inner(ui);
                }),
            };
        }
        HtmlElement::Button(button) => {
            let color = ui.style().visuals.widgets.active.bg_fill;

            let template = button.template();

            let content = template.render(entries);

            if ui.add(egui::Button::new(content).fill(color)).clicked() {
                // get button.evt_handlers Vec entry which matches EvtHandler.ty == OnClick
                if let Some(FuncAndArgs {
                    function: on_click,
                    args,
                }) = button.func_and_args(Action::OnClick)
                {
                    let args = {
                        let mut lock = plugin.lock().unwrap();
                        let scope = lock.store_mut().data_mut().scope_mut();
                        args.iter()
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
                    match lock.call(&on_click, args.as_slice()) {
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
        HtmlElement::Input(input) => {
            let is_password = input.is_password();

            // get the first TemplatPart::Dynamic from template.parts.iter()
            // otherwisse return early
            // since input doesn't have a closing tag, we need to take the template from elsewhere
            // <input value="{{name}}" data-on-change="handle_change(name)">
            let template = input.template();
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
                    if let Some(FuncAndArgs {
                        function: on_change,
                        args: func_args,
                    }) = input.func_and_args(Action::OnChange)
                    {
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

                            if let Ok(value) = lock.call(&on_change, args.as_slice()) {
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
        HtmlElement::Label { template }
        | HtmlElement::Span { template }
        | HtmlElement::Paragraph { template }
        | HtmlElement::Text { contents: template } => {
            let size = 16.0;
            let content = template.render(entries);

            ui.label(egui::RichText::new(content).size(size));
            ui.add_space(4.0);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
