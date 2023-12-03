use crate::{
    widget_description::{
        visuals::WidgetVisual,
        WidgetDescription,
        WidgetKind,
    },
    widgets::{
        SlotWidget,
        WidgetResponse,
    },
};
use egui::{
    vec2,
    Color32,
    InnerResponse,
    PointerButton,
    Pos2,
    Sense,
    Ui,
    Vec2,
};

pub struct Port {
    pub pos: Pos2,
    pub size: Vec2,
    pub visuals: Vec<WidgetVisual>,
}

impl SlotWidget for Port {
    fn pos(&self) -> Pos2 {
        self.pos
    }

    fn size(&self) -> Vec2 {
        vec2(16., 16.)
    }

    fn value(&self) -> f32 {
        0.0
    }

    fn show(&mut self, ui: &mut Ui) -> InnerResponse<WidgetResponse> {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click());

        ui.painter().debug_rect(rect, Color32::DEBUG_COLOR, "");
        let center = rect.center();
        if ui.is_rect_visible(rect) {
            for visual in &self.visuals {
                visual.show(ui, center, Sense::hover());
            }
        }
        let inner = if response.clicked_by(PointerButton::Primary) {
            WidgetResponse::AttemptConnection
        } else {
            WidgetResponse::None
        };
        InnerResponse::new(inner, response)
    }

    fn from_description(description: WidgetDescription) -> Option<Self>
    where
        Self: Sized,
    {
        let WidgetDescription {
            kind: WidgetKind::Port,
            name: _,
            pos,
            size,
            visuals,
        } = description
        else {
            return None;
        };

        let visuals = visuals.into_values().collect();
        Some(Self { pos, size, visuals })
    }
}
