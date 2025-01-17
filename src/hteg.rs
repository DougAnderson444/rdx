//! HTML to egui (HTEG) converter.and renderer in egui.
mod element_parser;
mod types;

use element_parser::Parser;
use html_to_egui::{Action, Selectors};
use rhai::CallFnOptions;
use types::{FuncAndArgs, HtmlElement};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use wasm_component_layer::Value;

use crate::layer::{Inner, Instantiator};
use crate::template::TemplatePart;
use crate::Error;

/// Parses the html and renders to egui for us.
#[derive(Clone)]
pub struct HtmlToEgui {
    parser: Parser,
    engine: Rc<RefCell<rhai::Engine>>,
    ast: rhai::AST,
}

impl HtmlToEgui {
    /// Create a new instance of the HtmlToEgui struct.
    /// This struct is used to parse HTML and render it into egui UI components.
    /// It takes a rhai::Engine and rhai::AST as arguments so that it can call rhai functions
    /// from plugins as needed.
    pub fn new(engine: Rc<RefCell<rhai::Engine>>, ast: rhai::AST) -> Self {
        Self {
            parser: Parser::default(),
            engine,
            ast,
        }
    }

    /// Parses the html text into a Vec of [scraper::html::Select] elements.
    /// Then renders the elements into egui UI components.
    pub fn parse_and_render<T: Inner + Clone + Send + Sync>(
        &mut self,
        ui: &mut egui::Ui,
        html: &str,
        plugin: Arc<parking_lot::Mutex<dyn Instantiator<T>>>,
    ) -> Result<(), Error> {
        let html_ast = self.parser.parse(html)?;
        self.render_element(ui, &html_ast, plugin.clone())?;
        Ok(())
    }

    /// Recurive function that walks the [scraper::ElementRef] and turns the
    /// HTML into egui UI components.
    fn render_element<T: Inner + Clone + Send + Sync>(
        &mut self,
        ui: &mut egui::Ui,
        element: &HtmlElement,
        plugin: Arc<parking_lot::Mutex<dyn Instantiator<T>>>,
    ) -> Result<(), Error> {
        // get the content scope values
        // converts the rhai::Dynamic value to a string
        let entries = {
            let lock = plugin.lock();
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
                    self.render_element(ui, child, plugin.clone())?;
                }
            }
            HtmlElement::Div {
                template: _, style, ..
            } => {
                let add_contents = |ui: &mut egui::Ui| {
                    if element.child_elements().is_some() {
                        for child in element.child_elements().unwrap() {
                            if let Err(e) = self.render_element(ui, child, plugin.clone()) {
                                tracing::error!("Error rendering child element: {:?}", e);
                            }
                        }
                    }
                };

                ui.set_max_width(ui.available_width());
                // Style the div as a flex row if the style has the FlexRow selector
                if style.get(&Selectors::FlexRow).is_some() {
                    ui.horizontal(add_contents);
                } else {
                    tracing::info!("Adding vertical div");
                }
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
                        let arguments = {
                            let mut lock = plugin.lock();
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
                            arguments,
                            arguments.len()
                        );

                        let mut lock = plugin.lock();
                        match lock.call(&on_click, arguments.as_slice()) {
                            Ok(res) => {
                                tracing::info!("on_click response {:?}", res);
                            }
                            Err(e) => {
                                tracing::error!("on_click Error {:?}", e);
                            }
                        }

                        // also call the same rhai function
                        // if it exists.
                        // if it doesn't exist, that;s ok, fail gracefully
                        // self.engine is the rhai::Engine
                        // self.ast is the rhai AST
                        // We're going to use Engine::call_fn_with_options because there's no need
                        // to compile the ast again, we've already got it.
                        // I don't think we can really do anything with the result here, we'll
                        // leave that for now.
                        let mut scope = lock.store_mut().data_mut().scope_mut();

                        let options = CallFnOptions::new()
                            .eval_ast(false) // do not re-evaluate the AST
                            .rewind_scope(false); // do not rewind the scope (i.e. keep new variables)

                        // Rhai functions are snake_case, but RDX functions can be any-case
                        // so we need to convert the function name to snake_case before calling it
                        let on_click = to_snake_case(on_click.as_str());

                        // get the arguments from rhai scope
                        let arguments: Vec<String> = args
                            .iter()
                            .map(|v| scope.get_value::<String>(v).unwrap_or_default())
                            .collect::<Vec<_>>();

                        // rhai functions should only change rhai Scope, not return anything
                        // Because, what would we do with the return value here?
                        tracing::info!("Calling on_click rhai function with args: {:?}", arguments);
                        match self.engine.borrow().call_fn_with_options::<rhai::Dynamic>(
                            options,
                            &mut scope,
                            &self.ast,
                            on_click.as_str(),
                            arguments,
                        ) {
                            Ok(result) => {
                                tracing::info!("on_click rhai function response: {:?}", result);
                            }
                            Err(e) => {
                                // It's ok though, we can fail gracefully
                                tracing::trace!("Error calling on_click rhai function: {:?}", e);
                            }
                        }

                        // print scope
                        //tracing::info!("Scope after on_click call_fn: {:?}", scope);
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
                let mut lock = plugin.lock();
                let mut scope = lock.store_mut().data_mut().scope_mut();

                if let Some(mut val) = scope.get_value::<String>(var_name.as_str()) {
                    let single_line = egui::TextEdit::singleline(&mut val)
                        .desired_width(400.0)
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

                                let mut lock = plugin.lock();

                                if let Ok(value) = lock.call(&on_change, args.as_slice()) {
                                    match value {
                                        Some(Value::String(_s)) => {
                                            // TODO: act on return value(s)?
                                        }
                                        Some(Value::Bool(_)) => {}
                                        _ => {}
                                    }
                                } else {
                                    tracing::error!(
                                        "Failed to call on_change function: {}",
                                        on_change
                                    );
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
}

/// Converts kebab-case and pascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut snake_case = String::new();
    let mut prev_is_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c == '-' || c == '_' {
            snake_case.push('_');
            prev_is_upper = false;
        } else if c.is_uppercase() {
            if i != 0 && !prev_is_upper {
                snake_case.push('_');
            }
            snake_case.push(c.to_ascii_lowercase());
            prev_is_upper = true;
        } else {
            snake_case.push(c);
            prev_is_upper = false;
        }
    }
    snake_case
}

#[cfg(test)]
mod tests {}
