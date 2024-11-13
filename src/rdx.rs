use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::layer::{Inner, LayerPlugin};
use crate::pest::{parse, Component};

use rhai::{Dynamic, Scope};
use tracing::error;
use wasm_component_layer::Value;

#[derive(Debug, Clone)]
struct State<'a> {
    scope: Arc<Mutex<Scope<'a>>>,
    egui_ctx: Option<egui::Context>,
}

impl<'a> State<'a> {
    pub fn new(ctx: egui::Context, scope: Arc<Mutex<Scope<'a>>>) -> Self {
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
        let mut scope = self.scope.lock().unwrap();
        scope.set_or_push(key, value.into());
        if let Some(egui_ctx) = &self.egui_ctx {
            tracing::info!("Requesting repaint");
            egui_ctx.request_repaint();
        } else {
            tracing::warn!("Egui context is not set");
        }
    }
}

pub struct RdxApp {
    scope: Arc<Mutex<Scope<'static>>>,
    components: Vec<Component>,
    rdx_source: String,
    plugins: HashMap<String, LayerPlugin<State<'static>>>,
}

impl Default for RdxApp {
    fn default() -> Self {
        let ctx = egui::Context::default();
        Self::new(ctx)
    }
}

impl RdxApp {
    pub fn new(ctx: egui::Context) -> Self {
        let scope = Arc::new(Mutex::new(Scope::new()));

        // set scope count var to 0
        scope.lock().unwrap().set_or_push("count", 0);

        let name = "counter";
        let wasm_bytes =
            include_bytes!("../target/wasm32-unknown-unknown/debug/counter.wasm").to_vec();
        let mut plugin = LayerPlugin::new(&wasm_bytes, State::new(ctx.clone(), scope.clone()));
        let rdx_source = plugin.call("load", &[]).unwrap();

        tracing::info!("RDX Source {:?}", rdx_source);

        let Value::String(rdx_source) = rdx_source else {
            panic!("RDX Source should be a string");
        };

        let components = parse(&rdx_source).unwrap();

        tracing::info!("Components {:?}", components);

        let mut plugins = HashMap::new();
        plugins.insert(name.to_string(), plugin);

        Self {
            scope,
            components,
            rdx_source: rdx_source.to_string(),
            plugins,
        }
    }
}

impl RdxApp {
    /// Return the source
    pub fn source(&self) -> &str {
        &self.rdx_source
    }

    pub fn components(&self) -> &Vec<Component> {
        &self.components
    }

    pub fn render_component(&mut self, ui: &mut egui::Ui, components: &Vec<Component>) {
        for component in components {
            match component {
                Component::Vertical { children, .. } => {
                    ui.vertical(|ui| {
                        self.render_component(ui, children);
                    });
                }
                Component::Horizontal { children, .. } => {
                    ui.horizontal(|ui| {
                        self.render_component(ui, children);
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
                            match self.plugins.get_mut("count").unwrap().call(on_click, &[]) {
                                Ok(res) => {
                                    tracing::info!("on_click response {:?}", res);
                                }
                                Err(e) => {
                                    error!("Error {:?}", e);
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
                        // let mut values = std::collections::HashMap::new();
                        // for part in &template.parts {
                        //     if let TemplatePart::Dynamic(key) = part {
                        //         if let Some(value) = self.scope.get_value::<String>(&key) {
                        //             values.insert(key.clone(), value.clone());
                        //         }
                        //     }
                        // }
                        template.render(self.scope.lock().unwrap().iter_raw())
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
            }
        }
    }
}
