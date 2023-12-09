use std::collections::BTreeMap;

use eframe::egui::{
    Context,
    Sense,
    Window,
};
use rack::widget_description::ModuleDescription;
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
