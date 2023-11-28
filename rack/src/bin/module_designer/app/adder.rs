use std::collections::BTreeMap;

use eframe::{
    egui::{ComboBox, TextEdit},
    epaint::Pos2,
};
use egui::{Context, Window};
use rack::{widget_description::WidgetDescription, widget_name, widget_prefabs};
use uuid::Uuid;

pub struct WidgetAdder {
    pub pos: Pos2,
    pub widget: Option<Uuid>,
    pub closing: bool,
    pub widgets: BTreeMap<Uuid, WidgetDescription>,
}

impl WidgetAdder {
    pub fn new(pos: Pos2) -> Result<Self, Box<dyn std::error::Error>> {
        let mut this = Self {
            pos,
            widget: None,
            closing: false,
            widgets: Default::default(),
        };
        for r in std::fs::read_dir("./prefabs")? {
            let f = r?;
            if f.file_type()?.is_file() {
                let prefabs = widget_prefabs(f.path())?;
                this.with_prefabs(prefabs);
            }
        }

        Ok(this)
    }

    pub fn with_prefabs(&mut self, prefabs: BTreeMap<Uuid, WidgetDescription>) {
        self.widgets.extend(prefabs);
    }

    pub fn selected_widget(&mut self) -> Option<&mut WidgetDescription> {
        self.widget.and_then(|uuid| self.widgets.get_mut(&uuid))
    }

    pub fn show(&mut self, ctx: &Context) {
        Window::new("New").resizable(false).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ComboBox::from_label("Widget")
                    .selected_text(widget_name(self.widget, &self.widgets))
                    .show_ui(ui, |ui| {
                        for uuid in self.widgets.keys() {
                            ui.selectable_value(
                                &mut self.widget,
                                Some(*uuid),
                                widget_name(Some(*uuid), &self.widgets),
                            );
                        }
                    });
                if let Some(widget) = self.selected_widget() {
                    TextEdit::singleline(&mut widget.name).show(ui);

                    for (k, v) in &mut widget.extra {
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
                }
            })
        });
    }
}
