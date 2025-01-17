#[allow(warnings)]
#[cfg_attr(rustfmt, rustfmt_skip)]
mod bindings;

use bindings::exports::component::plugin::run::Guest;

use std::sync::{LazyLock, Mutex};

static TODOS: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

struct Component;

impl Guest for Component {
    fn load() -> String {
        include_str!(concat!(env!("OUT_DIR"), "/todo.rhai")).to_string()
    }

    fn add_todo(todo: String) {
        let mut todos = TODOS.lock().unwrap();
        todos.push(todo);
    }

    fn todos() -> Vec<String> {
        let todos = TODOS.lock().unwrap();
        let t: Vec<String> = todos.iter().cloned().collect();
        t
    }

    fn register() -> Vec<String> {
        // This function will be available to us in Rhai.
        // It can only return Option, Bool, Array, String, or numbers.
        // No records (structs), enums (variants) as they are named,
        // and the host has no idea what your names are.
        // TODO: Grab names from the *.wasm wit to overcome this?
        vec!["todos".to_string()]
    }
}

bindings::export!(Component with_types_in bindings);
