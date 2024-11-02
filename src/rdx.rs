use crate::pest::{parse, Component};

use rhai::{Dynamic, Engine, Scope};
use tracing::{debug, error};

// #[derive(Clone, Debug)]
// pub struct Component {
//     name: String,
//     props: HashMap<String, String>,
//     children: Vec<Component>,
//     text_content: Option<String>,
// }

pub struct RdxApp {
    engine: Engine,
    scope: Scope<'static>,
    components: Component,
    rdx_source: String,
}

impl Default for RdxApp {
    fn default() -> Self {
        let engine = Engine::new();

        let scope = Scope::new();

        // Example RDX source with components
        let rdx_source = r#"
// Define a component render function
fn render() {
    let message = "Initial message";
    let count = 0;

    `
    <Horizontal>
        <Label size="large" color="blue">${message}</Label>
        
        <Label>The current count is: ${count}</Label>
        
        <Button onClick="increment" color="green">Increment</Button>
        <Button onClick="decrement" color="red">Decrement</Button>
        
        <Button color="gray">
            <Label size="small">This is a demo of RDX - Rhai + UI components!</Label>
        </Button>
    </Horizontal>
    `
}

// Define actions
fn increment() {
    count += 1;
    render()
}

fn decrement() {
    count -= 1;
    render()
}

// Initial render
render()
"#
        .to_string();

        Self {
            engine,
            scope,
            components: Component::Label {
                content: "Hello, RDX!".to_string(),
                props: Default::default(),
            },
            rdx_source,
        }
    }
}

impl RdxApp {
    /// Return the source
    pub fn source(&self) -> &str {
        &self.rdx_source
    }

    pub fn components(&self) -> &Component {
        &self.components
    }

    pub fn render_component(&self, ui: &mut egui::Ui, component: &Component) {
        match component {
            Component::Document { children } => {
                for child in children {
                    self.render_component(ui, child);
                }
            }
            Component::Vertical { children, .. } => {
                ui.vertical(|ui| {
                    for child in children {
                        self.render_component(ui, child);
                    }
                });
            }
            Component::Horizontal { children, .. } => {
                ui.horizontal(|ui| {
                    for child in children {
                        self.render_component(ui, child);
                    }
                });
            }
            Component::Button { content, props } => {
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
            } // "Heading" => {
              //     let size = match component.props.get("size").map(|s| s.as_str()) {
              //         Some("large") => 32.0,
              //         Some("medium") => 24.0,
              //         _ => 18.0,
              //     };
              //
              //     let color = match component.props.get("color").map(|s| s.as_str()) {
              //         Some("blue") => egui::Color32::from_rgb(100, 150, 255),
              //         Some("red") => egui::Color32::from_rgb(255, 100, 100),
              //         _ => ui.style().visuals.text_color(),
              //     };
              //
              //     ui.heading(
              //         egui::RichText::new(component.text_content.as_deref().unwrap_or(""))
              //             .size(size)
              //             .color(color),
              //     );
              //     ui.add_space(8.0);
              // }
              //
              // "Text" => {
              //     let size = match component.props.get("size").map(|s| s.as_str()) {
              //         Some("small") => 14.0,
              //         Some("large") => 18.0,
              //         _ => 16.0,
              //     };
              //
              //     ui.label(
              //         egui::RichText::new(component.text_content.as_deref().unwrap_or("")).size(size),
              //     );
              //     ui.add_space(4.0);
              // }
              //
              // "Button" => {
              //     let color = match component.props.get("color").map(|s| s.as_str()) {
              //         Some("green") => egui::Color32::from_rgb(100, 200, 100),
              //         Some("red") => egui::Color32::from_rgb(200, 100, 100),
              //         _ => ui.style().visuals.widgets.active.bg_fill,
              //     };
              //
              //     let text = component.text_content.as_deref().unwrap_or("");
              //     if ui.add(egui::Button::new(text).fill(color)).clicked() {
              //         if let Some(on_click) = component.props.get("onClick") {
              //             self.engine
              //                 .eval_with_scope::<Dynamic>(&mut self.scope.clone(), on_click)
              //                 .ok();
              //         }
              //     }
              //     ui.add_space(4.0);
              // }
              //
              // "Panel" => {
              //     let color = match component.props.get("color").map(|s| s.as_str()) {
              //         Some("gray") => egui::Color32::from_rgb(200, 200, 200),
              //         _ => ui.style().visuals.window_fill(),
              //     };
              //
              //     egui::Frame::none()
              //         .fill(color)
              //         .inner_margin(egui::Margin::same(8.0))
              //         .show(ui, |ui| {
              //             for child in &component.children {
              //                 self.render_component(ui, child);
              //             }
              //         });
              //     ui.add_space(8.0);
              // }
        }
    }

    pub fn update_components(&mut self) {
        debug!("Scope: {:#?}", self.scope);
        self.scope.push("count", 0);
        self.scope.push("message", "Hello, RDX!");

        tracing::info!("evaluating RDX source {:?}", self.scope);

        match self
            .engine
            .eval_with_scope::<Dynamic>(&mut self.scope, &self.rdx_source)
        {
            Ok(result) => {
                tracing::info!("eval result: {:?}", result);

                // parse the result as string and set self.components if parse is ok
                let s = result.to_string();
                let res: Component = parse(&s).unwrap();
                self.components = res;
            }
            Err(e) => {
                error!("Error updating components: {}", e);
            }
        }
    }
}

// impl eframe::App for Rdx {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             // Left panel with the RDX source code
//             ui.horizontal(|ui| {
//                 ui.add(
//                     egui::TextEdit::multiline(&mut self.rdx_source)
//                         .desired_width(ui.available_width() * 0.5)
//                         .desired_rows(30)
//                         .font(egui::TextStyle::Monospace),
//                 );
//
//                 ui.vertical(|ui| {
//                     if ui.button("Update").clicked() {
//                         self.update_components();
//                     }
//
//                     ui.add_space(20.0);
//
//                     // Render the components
//                     for component in &self.components {
//                         self.render_component(ui, component);
//                     }
//                 });
//             });
//         });
//     }
// }
