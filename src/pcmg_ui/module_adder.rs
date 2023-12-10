use std::collections::BTreeMap;

use eframe::egui::{
    Context,
    Sense,
    Window,
};
use rack::{
    visuals::VisualTheme,
    widget_description::ModuleDescription,
};
use uuid::Uuid;

pub struct ModuleAdder {
    pub selection: Option<Uuid>,
    pub modules: BTreeMap<Uuid, ModuleDescription>,
}

impl ModuleAdder {
    pub fn new(modules: BTreeMap<Uuid, ModuleDescription>) -> Self {
        Self {
            selection: None,
            modules,
        }
    }

    pub fn show(&mut self, ctx: &Context) -> bool {
        let mut closing = false;

        Window::new("Add module").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for (uuid, module) in self.modules.iter() {
                        ui.selectable_value(&mut self.selection, Some(*uuid), uuid.to_string());
                        let (r, _) = ui.allocate_exact_size(module.size.size(), Sense::hover());
                        for vis in module.visuals.values() {
                            vis.preview(
                                ui,
                                r.center() + vis.position.to_vec2(),
                                VisualTheme::default(),
                                0.0,
                            )
                        }
                    }
                    closing = ui.button("Add").clicked();
                });
                ui.vertical(|ui| {
                    if let Some(uuid) = self.selection {
                        let m = &self.modules[&uuid];
                        let (r, _) = ui.allocate_exact_size(m.size.size(), Sense::hover());
                        for w in m.visuals.values() {
                            w.preview(
                                ui,
                                r.center() + w.position.to_vec2(),
                                Default::default(),
                                0.0,
                            );
                        }
                    }
                });
            });
        });
        closing
    }
}
