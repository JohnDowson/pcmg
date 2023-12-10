use eframe::{
    egui::{
        PointerButton,
        Response,
        Sense,
        Ui,
    },
    emath::lerp,
    epaint::Pos2,
};
use egui::{
    Color32,
    InnerResponse,
    Vec2,
};

use crate::{
    visuals::{
        templates::WidgetTemplate,
        VisualComponent,
        VisualTheme,
    },
    widget_description::{
        KnobKind,
        WidgetKind,
    },
};

use super::{
    KnobRange,
    WidgetResponse,
};

fn calculate_value(value_range: KnobRange, angle: f32, angle_range: KnobRange) -> f32 {
    let normalized_angle = (angle - angle_range.start) / (angle_range.end - angle_range.start);
    lerp(value_range.start..=value_range.end, normalized_angle)
}

pub struct Knob {
    pub(crate) name: String,
    pub(crate) pos: Pos2,

    pub(crate) value: f32,
    value_range: KnobRange,

    angle: f32,
    angle_range: KnobRange,

    default_angle: f32,

    speed: f32,
    pub(crate) size: Vec2,

    visuals: Vec<VisualComponent>,
}

impl Knob {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        pos: Pos2,
        value_range: KnobRange,
        angle_range: KnobRange,
        default_pos: f32,
        speed: f32,
        size: Vec2,
        visuals: Vec<VisualComponent>,
    ) -> Self {
        let angle_range =
            KnobRange::from((angle_range.start.to_radians(), angle_range.end.to_radians()));

        let angle = lerp(angle_range, default_pos);
        let value = calculate_value(value_range, angle, angle_range);
        Self {
            name,
            pos,
            value,
            value_range,
            default_angle: angle,
            angle,
            angle_range,
            speed,
            size,
            visuals,
        }
    }

    fn allocate_space(&self, ui: &mut Ui) -> Response {
        ui.allocate_response(self.size, Sense::click_and_drag())
    }

    fn update(&mut self, ui: &mut Ui, theme: VisualTheme) -> Response {
        let old_value = self.value;
        let old_angle = self.angle;

        let mut res = self.allocate_space(ui);

        if res.clicked_by(PointerButton::Secondary) {
            self.angle = lerp(self.angle_range, self.default_angle);
        } else {
            let angle = if res.dragged() {
                let delta = res.drag_delta();
                let delta = delta.x - delta.y;
                let delta = delta * self.speed;

                if delta != 0.0 {
                    (old_angle + delta).clamp(self.angle_range.start, self.angle_range.end)
                } else {
                    old_angle
                }
            } else {
                old_angle
            };

            self.value = calculate_value(self.value_range, angle, self.angle_range);
            self.angle = angle;
        }

        self.draw(ui, &res, theme);

        res.changed = self.value != old_value;

        res
    }

    fn draw(&mut self, ui: &mut Ui, res: &Response, theme: VisualTheme) {
        let rect = res.rect;
        let center = rect.center();
        ui.painter().debug_rect(rect, Color32::GOLD, "");

        if ui.is_rect_visible(rect) {
            for visual in &self.visuals {
                visual.show_with_value(ui, center, theme, self.angle)
            }
        }
    }

    pub fn show(&mut self, ui: &mut Ui, theme: VisualTheme) -> InnerResponse<WidgetResponse> {
        let response = self.update(ui, theme);
        let wr = if response.changed() {
            WidgetResponse::Changed
        } else {
            WidgetResponse::None
        };
        InnerResponse::new(wr, response)
    }

    pub fn from_template(template: WidgetTemplate) -> Option<Self>
    where
        Self: Sized,
    {
        let WidgetTemplate {
            kind:
                WidgetKind::Knob(KnobKind {
                    angle_range,
                    value_range,
                    speed,
                }),
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

        Some(Self::new(
            name,
            pos,
            value_range,
            angle_range,
            0.0,
            speed,
            size,
            visuals,
        ))
    }
}
