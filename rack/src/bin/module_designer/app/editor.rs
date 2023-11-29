use egui::{
    ComboBox,
    Context,
    DragValue,
    Id,
    Slider,
    Window,
};
use rack::{
    container::StateValue,
    devices::Param,
    pos_drag_value,
    widget_description::{
        KnobKind,
        WidgetDescription,
        WidgetKind,
    },
};

pub struct WidgetEditor {
    extra_to_add: String,
    id: u16,
}

impl WidgetEditor {
    pub fn new(id: u16) -> Self {
        Self {
            extra_to_add: String::new(),
            id,
        }
    }

    pub fn show(
        &mut self,
        ctx: &Context,
        values: &[Param],
        w: &mut WidgetDescription,
    ) -> (bool, bool) {
        let mut delete = false;
        let mut closing = false;

        Window::new("Edit")
            .id(Id::new(self.id))
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut w.name);
                    });

                    pos_drag_value(ui, "Pos", &mut w.pos);

                    match &mut w.kind {
                        WidgetKind::Blinker => {}
                        WidgetKind::Knob(KnobKind {
                            value_range: (v_s, v_e),
                            angle_range: (a_s, a_e),
                            default_pos,
                            speed,
                        }) => {
                            ui.horizontal(|ui| {
                                ui.label("Value range");
                                ui.add(DragValue::new(v_s));
                                ui.add(DragValue::new(v_e));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Angle range");
                                ui.add(DragValue::new(a_s).clamp_range(0.0..=360.));
                                ui.add(DragValue::new(a_e).clamp_range(0.0..=360.));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Default position");
                                ui.add(Slider::new(default_pos, 0.0..=1.0));
                                ui.label("Speed");
                                ui.add(Slider::new(speed, 0.0..=1.0));
                            });
                        }
                        WidgetKind::InPort => {}
                        WidgetKind::OutPort => {}
                    }

                    ui.horizontal(|ui| {
                        ComboBox::from_label("Connected value")
                            .selected_text(
                                values
                                    .get(w.value)
                                    .map(ToString::to_string)
                                    .unwrap_or("None".into()),
                            )
                            .show_ui(ui, |ui| {
                                for (i, v) in values.iter().enumerate() {
                                    ui.selectable_value(&mut w.value, i, v.to_string());
                                }
                            });
                    });

                    ui.label("Extras");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.extra_to_add);
                        if ui.button("+").clicked() {
                            let extra = std::mem::take(&mut self.extra_to_add);
                            w.extra.insert(extra, StateValue::Float(0.0));
                        }
                    });

                    let mut delete_extra = None;
                    for (k, v) in &mut w.extra {
                        let mut edit = v.to_string();
                        ui.horizontal(|ui| {
                            ui.label(k);
                            ui.text_edit_singleline(&mut edit);
                            if ui.button("-").clicked() {
                                delete_extra = Some(k.clone());
                            }
                        });
                        if let Ok(nv) = edit.parse() {
                            *v = nv;
                        }
                    }
                    if let Some(d) = delete_extra {
                        w.extra.remove(&d);
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Ok").clicked() {
                            closing = true;
                        }
                        if ui.button("Delete").clicked() {
                            delete = true;
                            closing = true;
                        }
                    });
                })
            });
        (delete, closing)
    }
}
