use crate::{
    visuals::{
        templates::WidgetTemplate,
        VisualComponent,
        VisualTheme,
    },
    widget_description::WidgetKind,
    widgets::WidgetResponse,
};
use egui::{
    Color32,
    InnerResponse,
    PointerButton,
    Pos2,
    Sense,
    Ui,
    Vec2,
};

pub struct Port {
    pub name: String,
    pub pos: Pos2,
    pub size: Vec2,
    pub visuals: Vec<VisualComponent>,
}

impl Port {
    pub fn show(&mut self, ui: &mut Ui, theme: VisualTheme) -> InnerResponse<WidgetResponse> {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click());

        ui.painter().debug_rect(rect, Color32::DEBUG_COLOR, "");
        let center = rect.center();
        if ui.is_rect_visible(rect) {
            for visual in &self.visuals {
                visual.show(ui, center, theme);
            }
        }
        let inner = if response.clicked_by(PointerButton::Primary) {
            WidgetResponse::AttemptConnection
        } else if response.clicked_by(PointerButton::Secondary) {
            WidgetResponse::AttemptDisconnect
        } else {
            WidgetResponse::None
        };
        InnerResponse::new(inner, response)
    }

    pub fn from_template(template: WidgetTemplate) -> Option<Self>
    where
        Self: Sized,
    {
        let WidgetTemplate {
            kind: WidgetKind::Port,
            uuid: _,
            name,
            position: pos,
            size,
            components: visuals,
        } = template
        else {
            return None;
        };

        let visuals = visuals
            .into_values()
            .filter_map(|v| v.try_into().ok())
            .collect();

        Some(Self {
            name,
            pos,
            size,
            visuals,
        })
    }
}
