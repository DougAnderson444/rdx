use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::futures;
use crate::layer::{Inner, LayerPlugin};
use crate::pest::{parse, Component};
use crate::template::TemplatePart;

use rhai::{Dynamic, Scope};
use tracing::error;
use wasm_component_layer::Value;

#[derive(Debug)]
pub struct State<'a> {
    scope: Scope<'a>,
    egui_ctx: Option<egui::Context>,
}

impl<'a> State<'a> {
    pub fn new(ctx: egui::Context, scope: Scope<'a>) -> Self {
        Self {
            scope,
            egui_ctx: Some(ctx),
        }
    }
}

impl Inner for State<'_> {
    /// Updates the scope variable to the given value
    fn update(&mut self, key: &str, value: impl Into<Dynamic> + Copy) {
        tracing::info!("Updating key: {} with value: {:?}", key, value.into());
        self.scope.set_or_push(key, value.into());
        if let Some(egui_ctx) = &self.egui_ctx {
            tracing::info!("Requesting repaint");
            egui_ctx.request_repaint();
            // now that the rhai scope has been updated, we need to re-run
        } else {
            tracing::warn!("Egui context is not set");
        }
    }
}

/// The details of a plugin
pub struct PluginDeets {
    /// Reference counted so we can pass it into the rhai engine closure
    pub plugin: Arc<Mutex<LayerPlugin<State<'static>>>>,
    // Could be here for display purposes only, once it's compiled we're done using it.
    // rdx_source: String,
    engine: rhai::Engine,
    ast: Option<rhai::AST>,
}

impl PluginDeets {
    fn new(name: String, plugin: LayerPlugin<State<'static>>, rdx_source: String) -> Self {
        let mut engine = rhai::Engine::new();

        let plugin = Arc::new(Mutex::new(plugin));
        let plugin_clone = plugin.clone();
        let id = name.to_string();

        engine.register_fn("render", move |ctx: egui::Context, text: &str| {
            // Options are only Window, Area, CentralPanel, SidePanel, TopBottomPanel
            egui::Window::new(id.clone())
                .resizable(true)
                .show(&ctx, |ui| {
                    // dilemma here is: do you re-parse the RDX every time you render?
                    // if it's parsed once, where is the Component stored?
                    // and How do we refer to it?
                    // parse it once then store it in a cache for each RDX string?
                    // use std::cell::LazyCell (or LazyLock for sync)
                    if let Ok(components) = parse(text) {
                        render_component(ui, components, plugin_clone.clone());
                    }
                });
        });

        let ast = match engine.compile(&rdx_source) {
            Ok(ast) => Some(ast),
            Err(e) => {
                tracing::error!("Failed to compile RDX source: {:?}", e);
                None
            }
        };

        Self {
            plugin,
            engine,
            ast,
        }
    }

    /// Render this plugin's UI into the given ctx
    pub fn render_rhai(&mut self, ctx: egui::Context) {
        let mut scope = {
            let mut plugin = self.plugin.lock().unwrap();
            plugin.store.data_mut().scope.set_or_push("ctx", ctx);
            let scope = plugin.store.data().scope.clone();
            scope
        };

        if let Some(ast) = &self.ast {
            // Execute script
            if let Err(e) = self.engine.run_ast_with_scope(&mut scope, ast) {
                error!("Failed to execute script: {:?}", e);
            }
        }
    }
}

/// Render the components of this plugin
pub fn render_component(
    ui: &mut egui::Ui,
    components: Vec<Component>,
    plugin: Arc<Mutex<LayerPlugin<State<'static>>>>,
) {
    for component in components {
        match component {
            Component::Vertical { children, .. } => {
                ui.vertical(|ui| {
                    render_component(ui, children, plugin.clone());
                });
            }
            Component::Horizontal { children, .. } => {
                ui.horizontal(|ui| {
                    render_component(ui, children, plugin.clone());
                });
            }
            Component::Button {
                content,
                props,
                functions,
            } => {
                let color = match props.get("color").map(|s| s.as_str()) {
                    Some("green") => egui::Color32::from_rgb(100, 200, 100),
                    Some("red") => egui::Color32::from_rgb(200, 100, 100),
                    _ => ui.style().visuals.widgets.active.bg_fill,
                };

                let text = content.clone().unwrap_or("".to_string());
                if ui.add(egui::Button::new(&text).fill(color)).clicked() {
                    if let Some(on_click) = props.get("on_click") {
                        // if we had to call Rhai to execute the function:
                        //
                        // self.engine
                        //     .eval_with_scope::<Dynamic>(
                        //         &mut self.scope.lock().unwrap(),
                        //         on_click,
                        //     )
                        //     .ok();
                        tracing::debug!("On click {:?}", on_click);
                        let func_args = functions.get(on_click).unwrap();
                        tracing::debug!("Func args {:?}", func_args);
                        match plugin.lock().unwrap().call(on_click, &[]) {
                            Ok(res) => {
                                tracing::info!("on_click response {:?}", res);
                            }
                            Err(e) => {
                                error!("on_click Error {:?}", e);
                            }
                        }
                    }
                }
                ui.add_space(4.0);
            }
            Component::Label {
                content,
                props,
                template,
            } => {
                let size = match props.get("size").map(|s| s.as_str()) {
                    Some("small") => 14.0,
                    Some("large") => 18.0,
                    _ => 16.0,
                };

                let content = if let Some(template) = template {
                    let lock = plugin.lock().unwrap();
                    let state = lock.store.data();
                    // map the key, is_constant, valure to &str, &str iterator
                    let entries = state
                        .scope
                        .iter()
                        .map(|(k, _c, v)| (k, v.to_string()))
                        .collect::<Vec<_>>();
                    template.render(entries)
                } else {
                    content.to_string()
                };

                ui.label(egui::RichText::new(content).size(size));
                ui.add_space(4.0);
            }
            Component::Text { content, props } => {
                let size = match props.get("size").map(|s| s.as_str()) {
                    Some("small") => 14.0,
                    Some("large") => 18.0,
                    _ => 16.0,
                };

                ui.label(egui::RichText::new(content.clone()).size(size));
                ui.add_space(4.0);
            }
            Component::TextEdit {
                props,
                functions,
                template,
                ..
            } => {
                // 1. Get the variable from the template Dynamic String (there should only be one)
                // 2. Put the rhai::Scope value of that variable into the textEdit.
                // 3. on TextEdit changed(), update the rhai::Scope value of that variable

                // Variable name from template value
                // take the first Dynamic String from the template
                let var_name = template.as_ref().and_then(|t| {
                    t.parts.iter().find_map(|part| match part {
                        TemplatePart::Dynamic(s) => Some(s),
                        _ => None,
                    })
                });

                if let Some(var_name) = var_name {
                    // Get the value of the variable from the rhai::Scope
                    // Put the value into rhai::Scope as the value of the variable
                    // Can I just linkt he rhai scope variable to the TextEdit widget?
                    let mut lock = plugin.lock().unwrap();
                    let scope = &mut lock.store.data_mut().scope;

                    if let Some(mut val) = scope.get_value::<String>(var_name.as_str()) {
                        let response = ui.add(egui::TextEdit::singleline(&mut val));
                        if response.changed() {
                            // update the scope variable
                            scope.set_or_push(var_name.as_str(), val);

                            // also call the on_change function
                            if let Some(on_change) = props.get("on_change") {
                                if let Some(func_args) = functions.get(on_change) {
                                    let args = func_args
                                        .iter()
                                        .map(|v| Value::String(v.to_string().into()))
                                        .collect::<Vec<_>>();
                                    if let Ok(value) = lock.call(on_change, args.as_slice()) {
                                        match value {
                                            Some(Value::String(_s)) => {
                                                // TODO: set the scope variable to the returned value of on_change fn?
                                                // scope.set_or_push(var_name.as_str(), s);
                                            }
                                            Some(Value::Bool(_)) => {}
                                            _ => {}
                                        }
                                    } else {
                                        error!("Failed to call on_change function: {}", on_change);
                                    }
                                }
                            }
                        }
                    } else {
                        scope.set_or_push(var_name.as_str(), format!("inital {}", var_name));
                    }

                    // This doesn't work, see: https://github.com/rhaiscript/rhai/issues/933
                    // if let Some(var_ptr) = scope.get_value_mut::<String>(var_name.as_str()) {
                    //     ui.text_edit_singleline(var_ptr);
                    // } else {
                    //     tracing::error!("Failed to get var: {}", var_name);
                    //     scope.set_or_push(var_name.as_str(), format!("inital {}", var_name));
                    // }
                }

                ui.add_space(4.0);
            }
        }
    }
}
pub struct RdxApp {
    pub(crate) plugins: HashMap<String, PluginDeets>,
}

impl Default for RdxApp {
    fn default() -> Self {
        let ctx = egui::Context::default();
        Self::new(ctx)
    }
}

impl RdxApp {
    pub fn new(ctx: egui::Context) -> Self {
        let mut plugins = HashMap::new();
        for (name, wasm_bytes) in crate::BUILTIN_PLUGINS.iter() {
            let scope = Scope::new();

            // TODO: init from wasm logic somehow!
            // scope.set_or_push("count", 0);

            let mut plugin = LayerPlugin::new(wasm_bytes, State::new(ctx.clone(), scope.clone()));
            let rdx_source = plugin.call("load", &[]).unwrap();
            let Some(Value::String(rdx_source)) = rdx_source else {
                panic!("RDX Source should be a string");
            };
            plugins.insert(
                name.to_string(),
                PluginDeets::new(name.to_string(), plugin, rdx_source.to_string()),
            );
        }

        Self { plugins }
    }
}

impl RdxApp {}
