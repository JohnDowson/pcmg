use egui::{
    ComboBox,
    Context,
    DragValue,
    TextEdit,
    Window,
};
use rack::{
    pos_drag_value,
    vec_drag_value,
    widget_description::visuals::{
        WidgetVisual,
        WidgetVisualKind,
        WidgetVisualMode,
    },
};

pub struct VisualCreator {
    pub id: usize,
    label: &'static str,
}

impl VisualCreator {
    pub fn new(id: usize, label: &'static str) -> Self {
        Self { id, label }
    }

    pub fn show(&self, ctx: &Context, visual: &mut WidgetVisual) -> (bool, bool) {
        let mut delete = false;
        let mut closing = false;

        Window::new(self.label).resizable(false).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    let mut fill = visual.style.filled.is_some();
                    ui.checkbox(&mut fill, "Fill");
                    if fill {
                        let mut c = visual.style.filled.unwrap_or_default();
                        ui.color_edit_button_srgba(&mut c);
                        visual.style.filled = Some(c);
                    }

                    ui.label("Stroke");
                    ui.add(DragValue::new(&mut visual.style.stroke.width));
                    ui.color_edit_button_srgba(&mut visual.style.stroke.color);
                });

                ui.horizontal(|ui| {
                    ComboBox::from_label("Kind")
                        .selected_text(visual.kind.to_string())
                        .show_ui(ui, |ui| {
                            for kind in WidgetVisualKind::all() {
                                let s = kind.to_string();
                                ui.selectable_value(&mut visual.kind, kind, s);
                            }
                        });

                    ComboBox::from_label("Mode")
                        .selected_text(visual.mode.to_string())
                        .show_ui(ui, |ui| {
                            for mode in WidgetVisualMode::all() {
                                let s = mode.to_string();
                                ui.selectable_value(&mut visual.mode, mode, s);
                            }
                        });
                });

                pos_drag_value(ui, "Center", &mut visual.center);

                match &mut visual.kind {
                    WidgetVisualKind::Point => {}
                    WidgetVisualKind::Circle(r) => {
                        ui.horizontal(|ui| {
                            ui.label("Radius");
                            ui.add(DragValue::new(r));
                        });
                    }
                    WidgetVisualKind::Rect(size) => vec_drag_value(ui, "Size", size),
                    WidgetVisualKind::Line(end) => pos_drag_value(ui, "End", end),
                    WidgetVisualKind::Readout(size) => {
                        ui.horizontal(|ui| {
                            ui.label("y");
                            ui.add(DragValue::new(size).clamp_range(1.0..=64.));
                        });
                    }
                    WidgetVisualKind::Text(s) => {
                        ui.horizontal(|ui| {
                            ui.label("Text");
                            ui.add(TextEdit::singleline(s));
                        });
                    }
                    WidgetVisualKind::Symbol(c) => {
                        ui.horizontal(|ui| {
                            ui.label("Symbol");
                            let mut s = String::new();
                            s.push(*c);
                            ui.add(TextEdit::singleline(&mut s));
                            *c = s.chars().next().unwrap_or('-');
                        });
                    }
                }
                ui.horizontal(|ui| {
                    if ui.button("Finish").clicked() {
                        closing = true;
                    }
                    if ui.button("Delete").clicked() {
                        delete = true;
                        closing = true;
                    }
                })
            })
        });
        (delete, closing)
    }
}
