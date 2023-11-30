use crate::{
    container::module::SlotState,
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
    Rect,
    Response,
    Sense,
    Ui,
    Vec2,
};

pub struct Port {
    pub is_in: bool,
    pub value: usize,
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

    fn value(&self) -> usize {
        self.value
    }

    fn show(
        &mut self,
        ui: &mut Ui,
        _value: &mut f32,
        _extra_state: &mut SlotState,
    ) -> InnerResponse<WidgetResponse> {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click());

        ui.painter().debug_rect(rect, Color32::DEBUG_COLOR, "");
        let center = rect.center();
        if ui.is_rect_visible(rect) {
            for visual in &self.visuals {
                visual.show(ui, center, Sense::hover());
            }
        }
        let inner = if response.clicked_by(PointerButton::Primary) {
            if self.is_in {
                WidgetResponse::AttemptConnectionIn
            } else {
                WidgetResponse::AttemptConnectionOut
            }
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
            kind: kind @ WidgetKind::OutPort | kind @ WidgetKind::InPort,
            name: _,
            value,
            pos,
            size,
            visuals,
            extra: _,
        } = description
        else {
            return None;
        };

        let is_in = matches!(kind, WidgetKind::InPort);
        let visuals = visuals.into_values().collect();
        Some(Self {
            value,
            pos,
            is_in,
            size,
            visuals,
        })
    }
}
