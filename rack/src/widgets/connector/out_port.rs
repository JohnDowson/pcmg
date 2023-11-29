use crate::{
    container::SlotState,
    widget_description::{
        WidFull,
        WidgetDescription,
        WidgetKind,
    },
    widgets::SlotWidget,
};
use egui::{
    vec2,
    Pos2,
    Rect,
    Response,
    Sense,
    Ui,
    Vec2,
};

pub struct OutPort {
    pub id: WidFull,
    pub value: usize,
    pub pos: Pos2,
}

impl SlotWidget for OutPort {
    fn pos(&self) -> Pos2 {
        self.pos
    }

    fn size(&self) -> Vec2 {
        vec2(16., 16.)
    }

    fn value(&self) -> usize {
        self.value
    }

    fn show(&mut self, ui: &mut Ui, _value: &mut f32, _extra_state: &mut SlotState) -> Response {
        let p = ui.painter();
        let stroke = ui.visuals().widgets.active.bg_stroke;
        p.circle_stroke(self.pos(), self.size().x / 2., stroke);
        p.circle_stroke(self.pos(), self.size().x / 4., stroke);

        ui.allocate_rect(Rect::from_min_size(self.pos(), self.size()), Sense::hover())
    }

    fn from_description(id: WidFull, description: WidgetDescription) -> Option<Self>
    where
        Self: Sized,
    {
        let WidgetDescription {
            kind: WidgetKind::OutPort,
            name: _,
            value,
            pos,
            size: _,
            visuals: _,
            extra: _,
        } = description
        else {
            return None;
        };

        Some(Self { id, value, pos })
    }
}
