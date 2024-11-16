use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, LazyLock, Mutex},
};

use egui::ScrollArea;
use rhai::Dynamic;
use tracing_subscriber::fmt::format;

use crate::RdxApp;

/// Left Panel State
#[derive(serde::Deserialize, serde::Serialize)]
struct LeftPanelState {
    fraction: f32,
}

impl Default for LeftPanelState {
    fn default() -> Self {
        Self {
            fraction: 0.5, // Start with 50% height
        }
    }
}
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    #[serde(skip)]
    rdx: RdxApp,

    split_state: LeftPanelState,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            rdx: RdxApp::default(),
            split_state: LeftPanelState::default(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            tracing::debug!("Loading previous app state");
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        // set egui_ctx for the rdx app

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    ui.label("Demos");
                    // list all plugins by name here
                    let Self { rdx, .. } = self;
                    let RdxApp { plugins, .. } = rdx;
                    for (name, details) in plugins {
                        ui.toggle_value(&mut true, name);
                    }
                });
            });
        });

        let test_text = "Some test text";

        egui::SidePanel::left("inputs").show(ctx, |ui| {
            egui::TopBottomPanel::top("source_input")
                .resizable(true)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label("RDX Source");
                        ui.add(
                            egui::TextEdit::multiline(&mut test_text.to_owned())
                                .code_editor()
                                .desired_width(ui.available_width()),
                        );

                        // padding on the bottom
                        ui.add_space(20.0);
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label("State");
                    ui.add(
                        egui::TextEdit::multiline(&mut test_text.to_owned())
                            .code_editor()
                            .desired_width(ui.available_width()),
                    );

                    // padding on the bottom
                    ui.add_space(20.0);
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Output");
            ui.separator();

            let Self { rdx, .. } = self;
            let RdxApp { plugins, .. } = rdx;

            for (name, plugin) in plugins.iter_mut() {
                // tracing::debug!("Rendering plugin: {}", name);
                plugin.render_rhai(ctx.clone());
            }
        });
    }
}

fn try_rhai(ctx: egui::Context) {
    // Create Rhai engine
    let mut engine = rhai::Engine::new();

    let id = egui::Id::new("My Rhai Window");

    // Register egui functions
    engine.register_fn("render", move |ctx: &mut egui::Context, text: &str| {
        egui::Area::new(id).show(ctx, |ui| {
            ui.label(text);
            if ui.button(format!("Button: {}", text)).clicked() {
                // take some action here
            }
        });
    });

    // Create Rhai script
    let script = r#"
        render(ctx, "Hello from Rhai!");
    "#;

    // Compile script
    let ast = engine.compile(script).expect("Failed to compile script");

    // Create scope and add ctx
    let mut scope = rhai::Scope::new();
    scope.push("ctx", ctx);

    // Execute script
    engine
        .run_ast_with_scope(&mut scope, &ast)
        .expect("Failed to execute script");
}

/// Wrap the rhai update in a function to ensure that the lifetimes are covered
fn process_rhai_script(ui: &mut egui::Ui) {
    let shared_ui: crate::custom_types::SharedUi = Arc::new(Mutex::new(ui)); // <== borrowed data escapes because of the Mutex<&mut Ui>
                                                                             /**/
    // let dy = rhai::Dynamic::from(shared_ui.clone());

    let mut engine = rhai::Engine::new();
    engine.register_global_module(rhai::exported_module!(crate::custom_types::egui_api).into());
    engine.register_type::<egui::Response>();
    engine.register_type_with_name::<egui::Response>("Response");

    // FnMut not accepted here, as it mutates ui or doesn't live 'staticenough
    // let shared_ui_clone = shared_ui.clone();
    // engine.register_fn("test", move || {
    //     shared_ui_clone.lock().unwrap().label("Hello, world!");
    // });

    let mut scope = rhai::Scope::new();

    // Add the singleton command object into a custom Scope.
    // Constants, as a convention, are named with all-capital letters.
    // scope.push_constant("ui", shared_ui.clone());

    let script = r#"
        label(ui, "Hello, world!");
        let response = button(ui, "Click me!");
        label(ui, "Button clicked: ".to_owned() + &response.clicked.to_string());
    "#;

    // Compile script into AST
    let Ok(ast) = engine.compile(script) else {
        return;
    };

    // Run the compiled AST
    let Ok(_) = engine.run_ast_with_scope(&mut scope, &ast) else {
        return;
    };
}
