use std::collections::BTreeMap;

use eframe::egui::{
    ComboBox,
    TextEdit,
};
use egui::{
    Context,
    Window,
};
use rack::{
    devices::description::DeviceKind,
    visuals::templates::WidgetTemplate,
    widget_name,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct WidgetAdder {
    pub widget: Option<Uuid>,
    pub widgets: BTreeMap<Uuid, WidgetTemplate>,
}

impl WidgetAdder {
    pub fn new(widgets: BTreeMap<Uuid, WidgetTemplate>) -> Self {
        Self {
            widget: None,
            widgets,
        }
    }

    pub fn selected_widget(&mut self) -> Option<&mut WidgetTemplate> {
        self.widget.and_then(|uuid| self.widgets.get_mut(&uuid))
    }

    pub fn show(&mut self, ctx: &Context) -> bool {
        let mut closing = false;
        Window::new("New widget").resizable(false).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ComboBox::from_label("Widgets")
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

                    let r = ui.available_rect_before_wrap();
                    widget.preview(ui, r.center(), Default::default(), 0.0);

                    if ui.button("Add").clicked() {
                        closing = true;
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.widget = None;
                    closing = true
                }
            });
        });
        closing
    }
}

#[derive(Clone)]
pub struct DeviceAdder {
    pub device: usize,
    pub devices: Vec<DeviceKind>,
}

impl DeviceAdder {
    pub fn new() -> Self {
        Self {
            device: 0,
            devices: DeviceKind::all(),
        }
    }

    pub fn show(&mut self, ctx: &Context) -> (bool, bool) {
        let mut close = false;
        let mut selected = false;
        Window::new("New device").resizable(false).show(ctx, |ui| {
            for (di, dev) in self.devices.iter().enumerate() {
                ui.selectable_value(&mut self.device, di, dev.name());
            }

            ui.separator();

            ui.vertical_centered(|ui| {
                if ui.button("Add").clicked() {
                    close = true;
                    selected = true;
                }
                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });
        });
        (close, selected)
    }
}
