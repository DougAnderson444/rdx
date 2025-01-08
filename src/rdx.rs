#![allow(clippy::arc_with_non_send_sync)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::layer::{Inner, Instantiator, LayerPlugin, ScopeRef, ScopeRefMut};
use crate::pest::{parse, Component};
use crate::template::{Template, TemplatePart};

use rhai::{Dynamic, Scope};
use tracing::error;
use wasm_component_layer::Value;

#[cfg(target_arch = "wasm32")]
use send_wrapper::SendWrapper;

#[cfg(target_arch = "wasm32")]
use std::ops::Deref as _;

pub struct RdxApp {
    pub(crate) plugins: HashMap<String, PluginDeets<State>>,
}

impl Default for RdxApp {
    fn default() -> Self {
        Self::new(None)
    }
}

impl RdxApp {
    pub fn new(ctx: Option<egui::Context>) -> Self {
        let mut plugins = HashMap::new();
        for (name, wasm_bytes) in crate::BUILTIN_PLUGINS.iter() {
            // TODO: init from wasm logic somehow!
            // scope.set_or_push("count", 0);

            let mut plugin = LayerPlugin::new(wasm_bytes, State::new(ctx.clone()));
            let rdx_source = plugin.call("load", &[]).unwrap();
            let Some(Value::String(rdx_source)) = rdx_source else {
                panic!("RDX Source should be a string");
            };

            let plugin_mod = Arc::new(Mutex::new(plugin));

            plugins.insert(
                name.to_string(),
                PluginDeets::new(name.to_string(), plugin_mod, rdx_source.to_string()),
            );
        }

        Self { plugins }
    }
}

#[derive(Debug, Clone)]
pub struct State {
    scope: Arc<parking_lot::Mutex<Scope<'static>>>,
    egui_ctx: Option<egui::Context>,
}

impl State {
    pub fn new(ctx: Option<egui::Context>) -> Self {
        Self {
            scope: Arc::new(parking_lot::Mutex::new(Scope::new())),
            egui_ctx: ctx,
        }
    }
}

impl Inner for State {
    /// Updates the scope variable to the given value
    fn update(&mut self, key: &str, value: impl Into<Dynamic> + Clone) {
        self.scope.lock().set_or_push(key, value.into());
        if let Some(egui_ctx) = &self.egui_ctx {
            tracing::info!("Requesting repaint");
            egui_ctx.request_repaint();
            // now that the rhai scope has been updated, we need to re-run
        } else {
            tracing::warn!("Egui context is not set");
        }
    }

    fn scope(&self) -> ScopeRef {
        ScopeRef::Borrowed(self.scope.clone())
    }

    fn scope_mut(&mut self) -> ScopeRefMut {
        ScopeRefMut::Borrowed(self.scope.lock())
    }

    // into_scope with 'static lifetime'
    fn into_scope(self) -> rhai::Scope<'static> {
        self.scope.lock().clone()
    }
}

/// The plugin and all the details required to run it,
/// like the [rhai::Engine] and the [egui::Context]
pub struct PluginDeets<T: Inner + Send> {
    /// The name of the plugin
    name: String,
    /// Reference counted impl [Instantiator] so we can pass it into the rhai engine closure
    pub plugin: Arc<Mutex<dyn Instantiator<T>>>,
    /// The rhai engine
    pub engine: rhai::Engine,
    /// The AST of the RDX source
    ast: Option<rhai::AST>,
    /// The egui context, so we can `.show()` an [egui::Window]
    ctx: Option<egui::Context>,
}

impl<T: Inner + Clone + Send + Sync + 'static> PluginDeets<T> {
    /// pass any plugin that impls Instantiator
    pub fn new(name: String, plugin: Arc<Mutex<dyn Instantiator<T>>>, rdx_source: String) -> Self {
        let mut engine = rhai::Engine::new();

        engine.set_max_map_size(500); // allow object maps with only up to 500 properties

        // Compile the RDX source once ahead of time
        let ast = match engine.compile(&rdx_source) {
            Ok(ast) => Some(ast),
            Err(e) => {
                tracing::error!("Failed to compile RDX source: {:?}", e);
                None
            }
        };

        Self {
            name,
            plugin,
            engine,
            ast,
            ctx: None,
        }
    }

    /// Registers functions in the rhai Engine
    pub fn register_fn(&mut self) {
        #[cfg(target_arch = "wasm32")]
        let plugin_clone = SendWrapper::new(self.plugin.clone());
        #[cfg(not(target_arch = "wasm32"))]
        let plugin_clone = self.plugin.clone();

        let name = self.name.clone();
        let Some(ctx) = self.ctx.clone() else {
            tracing::warn!("Egui context is not set");
            return;
        };

        self.engine.register_fn("render", move |text: &str| {
            // Options are only Window, Area, CentralPanel, SidePanel, TopBottomPanel
            egui::Window::new(name.clone())
                .resizable(true)
                .show(&ctx, |ui| {
                    // TODO: We're re-parsing the RDX every time we render.
                    // This could be done using a HashMap of RDX strings to
                    // LazyLock to ensure it's only parsed once.

                    // unwrap the sendwrapper to get the plugin
                    #[cfg(target_arch = "wasm32")]
                    let plugin_clone = plugin_clone.deref();

                    if let Ok(components) = parse(text) {
                        render_component(ui, components, plugin_clone.clone());
                    } else {
                        tracing::error!(
                            "Failed to parse RDX source for plugin: {}, source {}",
                            name,
                            text
                        );
                    }
                });
        });

        //#[cfg(target_arch = "wasm32")]
        //let plugin_clone = SendWrapper::new(self.plugin.clone());
        //#[cfg(not(target_arch = "wasm32"))]
        //let plugin_clone = self.plugin.clone();
        //
        ////let plugin_clone = SendWrapper::new(self.plugin.clone());
        //self.engine.register_fn("unlocked", move || {
        //    // We need Fn (not FnOnce) because we're calling this function multiple times
        //    // So we need to clone again inside this closure so that we can call it multiple times
        //    let plugin_clone_clone = plugin_clone.clone();
        //
        //    futures::spawn(async move {
        //        let mut lock = plugin_clone_clone.lock().unwrap();
        //        let res = lock.call("unlocked", &[]).unwrap();
        //        tracing::info!("Locked response: {:?}", res);
        //        // if res is Some, unwrap and return it. If none, return false.
        //        res.map(|v| match v {
        //            Value::Bool(b) => b,
        //            _ => false,
        //        })
        //        .unwrap_or(false);
        //    });
        //});

        // Register all exported functions from the plugin
        // So they can be used by the Rhai script too
    }

    /// Render this plugin's UI into the given ctx
    pub fn render_rhai(&mut self, ctx: egui::Context) {
        // get the rhai scope, where the variables are stored
        // so we can pass it into the rhai engine
        // so the latest values are used in the script
        //let mut scope = {
        //    let plugin = self.plugin.lock().unwrap();
        //    //plugin.store.data_mut().scope.set_or_push("ctx", ctx);
        //    let scope = plugin.store().data().scope().clone();
        //    scope
        //};

        // sif self ctx is None, set it and call register(). This is a one-time thing
        if self.ctx.is_none() {
            self.ctx = Some(ctx.clone());
            self.register_fn();
        }

        if let Some(ast) = &self.ast {
            // Get the scope from the plugin and clone it
            let mut scope = {
                let Ok(plugin) = self.plugin.lock() else {
                    tracing::error!("Failed to lock plugin");
                    return;
                };

                plugin.store().data().clone().into_scope()
            };

            // Execute script.
            // Since the script returns `render(some_rdx_code)`, it will in turn
            // call register_fn("render") and render the UI.
            if let Err(e) = self.engine.run_ast_with_scope(&mut scope, ast) {
                error!("Failed to execute script: {:?}", e);
                // check if e matches  rhai::EvalAltResult::ErrorFunctionNotFound
                // if so, call register_fn() and try again
                if let rhai::EvalAltResult::ErrorFunctionNotFound(_, _) = e.as_ref() {
                    //self.register_fn();
                    //if let Err(e) = self.engine.run_ast_with_scope(&mut scope, ast) {
                    //    error!("Failed to execute script: {:?}", e);
                    //}
                }
            }
            //match self.engine.call_fn::<()>(&mut scope, ast, "tick", ()) {
            //    Ok(_) => tracing::info!("Tick function called"),
            //    Err(_) => tracing::error!("Failed to call tick function"),
            //}
        }
    }
}

/// Render the components of this plugin
pub fn render_component<T: Inner + Clone + Send + Sync>(
    ui: &mut egui::Ui,
    components: Vec<Component>,
    plugin: Arc<Mutex<dyn Instantiator<T>>>,
) {
    let content_from_template = |content: String, template: Option<Template>| {
        if let Some(template) = template {
            let lock = plugin.lock().unwrap();
            let state = lock.store().data();
            // map the key, is_constant, valure to &str, &str iterator
            let scope = state.clone().into_scope();
            let entries = scope
                .iter()
                .map(|(k, _c, v)| (k, v.to_string()))
                .collect::<Vec<_>>();

            template.render(entries)
        } else {
            content.to_string()
        }
    };

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

                        let args = {
                            let mut lock = plugin.lock().unwrap();
                            let scope = lock.store_mut().data_mut().scope_mut();
                            func_args
                                .iter()
                                .map(|v| {
                                    Value::String(
                                        scope.get_value::<String>(v).unwrap_or_default().into(),
                                    )
                                })
                                .collect::<Vec<_>>()
                        };

                        tracing::debug!("rt'd args {:?}", args);

                        let mut lock = plugin.lock().unwrap();
                        match lock.call(on_click, args.as_slice()) {
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

                let content = content_from_template(content, template);

                ui.label(egui::RichText::new(content).size(size));
                ui.add_space(4.0);
            }
            Component::Text {
                content,
                props,
                template,
            } => {
                let content = content_from_template(content, template);

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

                // check whether this is a password TextEdit or not
                let is_password = props.get("password").map(|s| s.as_str()) == Some("true");

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
                    let mut scope = lock.store_mut().data_mut().scope_mut();

                    if let Some(mut val) = scope.get_value::<String>(var_name.as_str()) {
                        let single_line = egui::TextEdit::singleline(&mut val)
                            .desired_width(f32::INFINITY)
                            .password(is_password);
                        let response = ui.add(single_line);
                        if response.changed() {
                            // update the scope variable
                            scope.set_or_push(var_name.as_str(), val.clone());

                            // also call the on_change function
                            if let Some(on_change) = props.get("on_change") {
                                if let Some(func_args) = functions.get(on_change) {
                                    let args = func_args
                                        .iter()
                                        .map(|v| {
                                            Value::String(
                                                scope
                                                    .get_value::<String>(v)
                                                    .unwrap_or_default()
                                                    .into(),
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
                                        error!("Failed to call on_change function: {}", on_change);
                                    }
                                }
                            }
                        }
                    } else {
                        scope.set_or_push(var_name.as_str(), var_name.to_string());
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

#[cfg(test)]
mod tests {
    use super::*;

    // test calling a tick() function in the rhai script
    #[test]
    fn test_tick() {
        let mut engine = rhai::Engine::new();
        let mut scope = Scope::new();

        // dummy render
        engine.register_fn("render", |_: egui::Context, _: &str| {});

        engine.on_print(|s| eprint!("{}", s));

        let rdx_source = r#"
        let interval = 1000; // 1 second


        fn tick() {
            print("*** tick ***");
        }

        // call the system function `render` on the template with the ctx from scope
        render(ctx, `
            <Vertical>
                <Label>Seconds since unix was invented: {{datetime}}</Label>
            </Vertical>
        `);
        "#;

        let ast = engine.compile(rdx_source).unwrap();

        //// Execute script
        if let Err(e) = engine.run_ast_with_scope(&mut scope, &ast) {
            error!("Failed to execute script: {:?}", e);
        }

        // Call the tick function
        if let Err(e) = engine.call_fn::<()>(&mut scope, &ast, "tick", ()) {
            error!("Failed to call tick function: {:?}", e);
        }
    }
}
