use egui::{
    InnerResponse,
    Sense,
    Ui,
};
use emath::{
    Pos2,
    Vec2,
};

use crate::{
    module_description::{
        ToggleKind,
        WidgetKind,
    },
    visuals::{
        templates::WidgetTemplate,
        VisualComponent,
        VisualTheme,
    },
};

use super::WidgetResponse;

pub struct Toggle {
    pub(crate) name: String,
    pub(crate) pos: Pos2,
    pub(crate) size: Vec2,

    on: f32,
    off: f32,
    state: bool,

    visuals: Vec<VisualComponent>,
}

impl Toggle {
    pub fn new(
        name: String,
        pos: Pos2,
        size: Vec2,
        on: f32,
        off: f32,
        visuals: Vec<VisualComponent>,
    ) -> Self {
        Self {
            name,
            pos,
            size,
            on,
            off,
            state: false,
            visuals,
        }
    }

    pub fn value(&self) -> f32 {
        if self.state {
            self.on
        } else {
            self.off
        }
    }

    pub fn show(&mut self, ui: &mut Ui, theme: VisualTheme) -> InnerResponse<WidgetResponse> {
        let old_state = self.state;
        let mut res = ui.allocate_response(self.size, Sense::click());

        if res.clicked() {
            self.state = !self.state;
        }

        let rect = res.rect;
        let center = rect.center();
        if ui.is_rect_visible(rect) {
            for visual in &self.visuals {
                let value = if self.state { 1. } else { 0. };
                visual.show_with_value(ui, center, theme, value);
            }
        }

        let wr = if old_state == self.state {
            res.changed = false;
            WidgetResponse::None
        } else {
            res.changed = true;
            WidgetResponse::Changed
        };

        InnerResponse::new(wr, res)
    }

    pub(crate) fn from_template(template: WidgetTemplate) -> Option<Self> {
        let WidgetTemplate {
            kind: WidgetKind::Toggle(ToggleKind { on, off }),
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
        Some(Self::new(name, pos, size, on, off, visuals))
    }
}
