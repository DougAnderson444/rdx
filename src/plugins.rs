use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;

use bindgen::component::plugin::types::Event;
use eframe::egui::{self};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{DirPerms, FilePerms, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

use crate::error::Error;

pub(crate) mod bindgen {
    wasmtime::component::bindgen!();
}

pub trait Inner {}

/// Struct to hold the data we want to pass in
/// plus the WASI properties in order to use WASI
pub struct MyCtx<T: Inner> {
    /// This data can be accessed from [Store] by using the data method(s)
    #[allow(dead_code)]
    inner: T,
    wasi_ctx: Context,
}

// plugins::MyCtx<T>: plugins::bindgen::PluginWorldImports
impl<T: Inner> bindgen::PluginWorldImports for MyCtx<T> {
    fn emit(&mut self, evt: Event) {
        todo!()
    }
}

impl<T: Inner> Deref for MyCtx<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Inner> DerefMut for MyCtx<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Inner + Send + Clone> bindgen::component::plugin::types::Host for MyCtx<T> {}

struct Context {
    table: ResourceTable,
    wasi: WasiCtx,
}

// We need to impl to be able to use the WASI linker add_to_linker
impl<T: Inner + Send> WasiView for MyCtx<T> {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.wasi_ctx.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx.wasi
    }
}

/// [Environment] struct to hold the engine and Linker
#[derive(Clone)]
pub struct Environment<T: Inner + Clone> {
    engine: Engine,
    linker: Arc<Linker<MyCtx<T>>>,
    vars: Option<Vec<(String, String)>>,
    host_path: PathBuf,
}

impl<T: Inner + Send + Clone> Environment<T> {
    /// Creates a new [Environment]
    pub fn new(host_path: PathBuf) -> Result<Self, Error> {
        let mut config = Config::new();
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
        config.wasm_component_model(true);
        config.async_support(false);

        let engine = Engine::new(&config).unwrap();
        let mut linker = Linker::new(&engine);

        bindgen::PluginWorld::add_to_linker(&mut linker, |state: &mut MyCtx<T>| state)?;

        // add wasi io, filesystem, clocks, cli_base, random, poll
        wasmtime_wasi::add_to_linker_sync(&mut linker)?;

        Ok(Self {
            engine,
            linker: Arc::new(linker),
            vars: None,
            host_path,
        })
    }

    /// Sets environment variables. When used in with a plugin,
    pub fn with_vars(mut self, vars: Vec<(String, String)>) -> Self {
        self.vars = Some(vars);
        self
    }
}

/// Extension struct to hold the wasm extension files
pub struct Plugin<T: Inner> {
    /// The built bindings for the wasm extensions
    pub instance: bindgen::PluginWorld,

    /// The store to run the wasm extensions
    store: Store<MyCtx<T>>,
}

impl<T: Inner + Send + Clone> Plugin<T> {
    /// Creates and instantiates a new [Plugin]
    pub fn new(
        env: Environment<T>,
        name: &str,
        wasm_bytes: &[u8],
        state: T,
    ) -> Result<Self, Error> {
        let component = Component::from_binary(&env.engine, wasm_bytes)?;

        // ensure the HOST PATH / name exists, if not, create it
        let host_plugin_path_name = env.host_path.join(name);
        // tracing::info!("Creating host plugin path: {:?}", host_plugin_path_name);
        std::fs::create_dir_all(&host_plugin_path_name)?;

        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_stdout()
            .envs(&env.vars.unwrap_or_default())
            .preopened_dir(
                &host_plugin_path_name,
                ".",
                DirPerms::all(),
                FilePerms::all(),
            )?
            .build();

        let data = MyCtx {
            inner: state,
            wasi_ctx: Context {
                table: ResourceTable::new(),
                wasi,
            },
        };
        let mut store = Store::new(&env.engine, data);

        // get a &mut linker by deref muting it
        // let lnkr = &mut *env.linker.clone();
        // let lnkr = &mut (*env.linker).clone();
        // bindgen::ExtensionWorld::add_to_linker(lnkr, |state: &mut MyCtx<T>| state)?;

        let instance = bindgen::PluginWorld::instantiate(&mut store, &component, &env.linker)?;

        Ok(Self { instance, store })
    }

    /// Access to the inner state, the T in self.store: Store<MyCtx<T>>.
    pub fn state(&self) -> &T {
        &self.store.data().inner
    }

    /// Loads the RDX from the component
    pub fn load_rdx(&mut self) -> Result<String, Error> {
        let rdx = self.instance.call_load(&mut self.store)?;
        Ok(rdx)
    }
}
