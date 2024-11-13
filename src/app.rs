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

    source: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            rdx: RdxApp::default(),
            split_state: LeftPanelState::default(),
            source: "".to_string(),
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

        tracing::debug!("Creating new app state");
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            rdx: RdxApp::new(cc.egui_ctx.clone()),
            split_state: LeftPanelState::default(),
            source: "".to_string(),
        }
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
            ui.label("Demos");
            ui.label("This is a placeholder for a right panel.");
            ui.label("It could contain e.g. a list of entities.");
        });

        egui::SidePanel::left("inputs").show(ctx, |ui| {
            egui::TopBottomPanel::top("source_input")
                .resizable(true)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label("RDX Source");
                        ui.add(
                            egui::TextEdit::multiline(&mut self.source)
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
                        egui::TextEdit::multiline(&mut self.source)
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

            // Render the components
            let all_components = self.rdx.components().clone();
            self.rdx.render_component(ui, &all_components);
        });

        // egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        //     // Left panel with the RDX source code
        //     ui.horizontal(|ui| {
        //         ui.add(
        //             egui::TextEdit::multiline(&mut self.rdx.source())
        //                 .desired_width(ui.available_width() * 0.5)
        //                 .desired_rows(30)
        //                 .font(egui::TextStyle::Monospace),
        //         );
        //
        //         ui.vertical(|ui| {
        //             if ui.button("Update").clicked() {
        //                 tracing::info!("Updating components");
        //                 // self.rdx.update_components();
        //                 ctx.request_repaint();
        //             }
        //
        //             ui.add_space(20.0);
        //         });
        //     });
        // });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
