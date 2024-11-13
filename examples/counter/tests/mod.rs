// name of the world in the .wit file
mod bindgen {
    wasmtime::component::bindgen!();
}

use bindgen::component::plugin::types::Event;
use std::{
    env,
    path::{Path, PathBuf},
};
use thiserror::Error;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

//use crate::bindgen::exports::component::extension::handlers;

struct MyCtx {
    count: i32,
    table: ResourceTable,
    ctx: WasiCtx,
}

impl WasiView for MyCtx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

impl bindgen::component::plugin::types::Host for MyCtx {}

impl bindgen::component::plugin::host::Host for MyCtx {
    fn emit(&mut self, _evt: Event) {
        // update Rhai state,
        self.count += 1;
    }
}

#[derive(Error, Debug)]
pub enum TestError {
    /// From String
    #[error("Error message {0}")]
    Stringified(String),

    /// From Wasmtime
    #[error("Wasmtime: {0}")]
    Wasmtime(#[from] wasmtime::Error),

    /// From VarError
    #[error("VarError: {0}")]
    VarError(#[from] std::env::VarError),

    /// From io
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    // // From<bindgen::component::extension::types::Error>
    // #[error("Error: {0}")]
    // BindgenError(bindgen::component::plugin::types::Error),
}

impl From<String> for TestError {
    fn from(s: String) -> Self {
        TestError::Stringified(s)
    }
}

/// Utility function to get the workspace dir
pub fn workspace_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}

#[cfg(test)]
mod test_mod_echo {

    use super::*;

    use wasmtime::AsContextMut;
    use wasmtime_wasi::{DirPerms, FilePerms};

    const HOST_PATH: &str = "./tests/exts";
    const GUEST_PATH: &str = ".";

    #[test]
    fn test_initial_load() -> wasmtime::Result<(), TestError> {
        // get the target/wasm32-wasi/debug/CARGO_PKG_NAME.wasm file
        let pkg_name = std::env::var("CARGO_PKG_NAME")?.replace('-', "_");
        let workspace = workspace_dir();
        let wasm_path = format!("target/wasm32-unknown-unknown/debug/{}.wasm", pkg_name);
        let wasm_path = workspace.join(wasm_path);

        let mut config = Config::new();
        config.cache_config_load_default()?;
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
        config.wasm_component_model(true);

        let engine = Engine::new(&config)?;
        let component = Component::from_file(&engine, &wasm_path)?;

        let mut linker = Linker::new(&engine);
        // link imports like get_seed to our instantiation
        bindgen::PluginWorld::add_to_linker(&mut linker, |state: &mut MyCtx| state)?;
        // link the WASI imports to our instantiation
        wasmtime_wasi::add_to_linker_sync(&mut linker)?;

        // ensure the HOST_PATH exists, if not, create it
        std::fs::create_dir_all(HOST_PATH)?;

        let table = ResourceTable::new();
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_stdout()
            .args(&[""])
            .preopened_dir(HOST_PATH, GUEST_PATH, DirPerms::all(), FilePerms::all())?
            .build();

        let state = MyCtx {
            table,
            ctx: wasi,
            count: 0,
        };
        let mut store = Store::new(&engine, state);

        let bindings = bindgen::PluginWorld::instantiate(&mut store, &component, &linker)?;

        let c = bindings
            .component_plugin_run()
            .call_increment(&mut store)
            .unwrap();

        assert_eq!(c, 1);
        // again

        let c_2 = bindings
            .component_plugin_run()
            .call_increment(&mut store)
            .unwrap();

        // store should have been updated too
        let state = store.data().count;
        assert_eq!(state, 2);

        assert_eq!(c_2, 2);

        // should be able to get_func "increment" behind the exported interface provider and call it.
        let name = "increment";

        let instance = linker
            .instantiate(store.as_context_mut(), &component)
            .unwrap();

        /**/
        let export_index = instance
            .get_export(&mut store, None, "component:plugin/run")
            .unwrap();

        let export_index = instance
            .get_export(&mut store, Some(&export_index), name)
            .unwrap();

        let increment = *instance
            .get_typed_func::<(), (i32,)>(&mut store, &export_index)?
            .func();
        let callee =
            unsafe { wasmtime::component::TypedFunc::<(), (i32,)>::new_unchecked(increment) };
        let (ret0,) = callee.call(store.as_context_mut(), ())?;
        callee.post_return(store.as_context_mut())?;

        assert_eq!(ret0, 1);

        let (ret1,) = callee.call(store.as_context_mut(), ())?;
        callee.post_return(store.as_context_mut())?;

        assert_eq!(ret1, 2);

        // let current = store.data().count;

        // For direct (no interace exports):

        // let func = instance.get_func(&mut store, name).unwrap();
        // let mut results = vec![Val::S32(0)];
        // func.call(&mut store, &[], &mut results).unwrap();
        //
        // assert_eq!(results[0], Val::S32(3));
        //
        // // post_return, so we can call it again
        // func.post_return(&mut store).unwrap();
        //
        // let mut results = vec![Val::S32(0)];
        // func.call(&mut store, &[], &mut results).unwrap();
        //
        // // post_return, so we can call it again
        // func.post_return(&mut store).unwrap();
        //
        // assert_eq!(results[0], Val::S32(4));

        Ok(())
    }
}
