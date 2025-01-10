#![allow(clippy::arc_with_non_send_sync)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::hteg::parse_and_render;
use crate::layer::{Inner, Instantiator, LayerPlugin, ScopeRef, ScopeRefMut};

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

        self.engine.register_fn("render", move |html: &str| {
            // Options are only Window, Area, CentralPanel, SidePanel, TopBottomPanel
            egui::Window::new(name.clone())
                .resizable(true)
                .show(&ctx, |ui| {
                    // [browser]: unwrap the sendwrapper to get the plugin
                    #[cfg(target_arch = "wasm32")]
                    let plugin_clone = plugin_clone.deref();

                    if let Err(e) = parse_and_render(ui, html, plugin_clone.clone()) {
                        tracing::error!(
                            "Failed to parse RDX source for plugin: {}, source {} with error: {:?}",
                            name,
                            html,
                            e
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
