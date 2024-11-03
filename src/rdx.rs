use crate::pest::{parse, Component};
use crate::plugins::{Environment, Inner, Plugin};
use crate::utils;

use rhai::{Dynamic, Engine, Scope};
use tracing::{debug, error};

#[derive(Debug, Clone, Default)]
struct State {
    count: i32,
}

impl Inner for State {}

pub struct RdxApp {
    engine: Engine,
    scope: Scope<'static>,
    components: Vec<Component>,
    rdx_source: String,
}

impl Default for RdxApp {
    fn default() -> Self {
        let env: Environment<State> = Environment::new("./plugin_path".into()).unwrap();

        let name = "counter";
        let wasm_path = utils::get_wasm_path(name).unwrap();
        let wasm_bytes = std::fs::read(wasm_path.clone()).unwrap();
        let mut plugin = Plugin::new(env.clone(), name, &wasm_bytes, State::default()).unwrap();
        let rdx_source = plugin.load_rdx().unwrap();
        let components = parse(&rdx_source).unwrap();

        let engine = Engine::new();

        let scope = Scope::new();

        Self {
            engine,
            scope,
            components,
            rdx_source,
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

    pub fn render_component(&self, ui: &mut egui::Ui, components: &Vec<Component>) {
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
                        if let Some(on_click) = props.get("onClick") {
                            self.engine
                                .eval_with_scope::<Dynamic>(&mut self.scope.clone(), on_click)
                                .ok();
                        }
                    }
                    ui.add_space(4.0);
                }
                Component::Label { content, props } => {
                    let size = match props.get("size").map(|s| s.as_str()) {
                        Some("small") => 14.0,
                        Some("large") => 18.0,
                        _ => 16.0,
                    };

                    ui.label(egui::RichText::new(content.clone()).size(size));
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

    pub fn update_components(&mut self) {
        self.scope.push("count", 0);
        self.scope.set_or_push("message", "Hello, RDX!");

        tracing::info!("evaluating RDX source {:?}", self.scope);

        match self
            .engine
            .eval_with_scope::<Dynamic>(&mut self.scope, &self.rdx_source)
        {
            Ok(result) => {
                // parse the result as string and set self.components if parse is ok
                let s = result.to_string();
                let res = parse(&s).unwrap();
                self.components = res;
            }
            Err(e) => {
                error!("Error updating components: {}", e);
            }
        }
    }
}
