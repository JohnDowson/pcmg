use eframe::{
    egui::{ComboBox, TextEdit, Widget},
    epaint::Pos2,
};
use rack::widget_description::{Wid, WidgetDescription, WidgetKind};

pub struct WidgetAdder {
    pub pos: Pos2,
    pub prefab: WidgetDescription,
    pub closing: bool,
    pub prefabs: Vec<WidgetDescription>,
}

impl WidgetAdder {
    pub fn new(pos: Pos2) -> Self {
        Self {
            pos,
            prefab: WidgetDescription::new(
                WidgetKind::Knob,
                Wid(0),
                String::new(),
                pos,
                Default::default(),
            ),
            closing: false,
            prefabs: Vec::new(),
        }
    }

    pub fn with_prefabs(&mut self, prefabs: Vec<WidgetDescription>) {
        self.prefabs = prefabs;
    }
}

impl Widget for &mut WidgetAdder {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical_centered(|ui| {
            ComboBox::from_label("Prefab")
                .selected_text(&self.prefab.name)
                .show_ui(ui, |ui| {
                    for prefab in &self.prefabs {
                        ui.selectable_value(&mut self.prefab, prefab.clone(), &prefab.name);
                    }
                });

            TextEdit::singleline(&mut self.prefab.name).show(ui);

            for (k, v) in &mut self.prefab.extra {
                let mut edit = v.to_string();
                ui.horizontal(|ui| {
                    ui.label(k);
                    ui.add(TextEdit::singleline(&mut edit))
                });
                if let Ok(nv) = edit.parse() {
                    *v = nv;
                }
            }

            if ui.button("Add").clicked() {
                self.closing = true
            }
        })
        .response
    }
}
