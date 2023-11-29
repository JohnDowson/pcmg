use egui::{
    ComboBox,
    Context,
    DragValue,
    TextEdit,
    Vec2,
    Window,
};
use rack::{
    vec_drag_value,
    widget_description::WidgetKind,
};
use uuid::Uuid;

pub struct WidgetCreator {
    pub uuid: Uuid,
    pub closing: bool,

    pub kind: WidgetKind,
    pub name: String,
    pub size: Vec2,
}

impl WidgetCreator {
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();

        Self {
            uuid,
            closing: false,
            kind: Default::default(),
            name: Default::default(),
            size: Default::default(),
        }
    }

    pub fn show(&mut self, ctx: &Context) {
        Window::new("New").resizable(false).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.add(TextEdit::singleline(&mut self.name));
                });

                ComboBox::from_label("Kind")
                    .selected_text(self.kind.to_string())
                    .show_ui(ui, |ui| {
                        for kind in WidgetKind::all() {
                            ui.selectable_value(&mut self.kind, kind, kind.to_string());
                        }
                    });

                vec_drag_value(ui, "Size", &mut self.size);

                if ui.button("Finish").clicked() {
                    self.closing =
                        self.size.x >= 1.0 && self.size.y >= 1.0 && !self.name.is_empty();
                }
            })
        });
    }
}
