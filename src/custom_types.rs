//! Import this [egui_api] module, then do:
//!
//! engine.register_global_module(exported_module!(egui_api).into());
//!
//! We also need to register the [egui::Response] using the following code:
//! engine.register_type::<egui::Response>();
//! engine.register_type_with_name::<egui::Response>("Response")
//!
//! Then Push constant command object into custom scope and run AST
//!
//! let mut scope = Scope::new();
//!
//! // Add the singleton command object into a custom Scope.
//! // Constants, as a convention, are named with all-capital letters.
//! scope.push_constant("BUNNY", bunny.clone());
//!
//! // Run the compiled AST
//! engine.run_ast_with_scope(&mut scope, &ast)?;

use std::sync::{Arc, Mutex};

use rhai::plugin::*;

/// Shared type
pub type SharedUi<'a> = Arc<Mutex<&'a mut egui::Ui>>;

// Remember to put 'pure' on all functions, or they'll choke on constants!
#[export_module]
pub mod egui_api {
    use egui::Response;

    // Custom type 'SharedBunny' will be called 'EnergizerBunny' in scripts
    // pub type Ui<'a> = SharedUi<'a>;

    // This constant is also available to scripts
    pub const MAX_SPEED: i64 = 100;

    /// Label
    #[rhai_fn(pure)]
    pub fn label<'a>(ui: &'a mut SharedUi, text: &str) {
        ui.lock().unwrap().label(text);
    }

    /// Button
    #[rhai_fn(pure)]
    pub fn button<'a>(ui: &'a mut SharedUi, text: &str) -> Response {
        ui.lock().unwrap().button(text)
    }
}
