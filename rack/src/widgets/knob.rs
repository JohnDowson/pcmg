use eframe::{
    egui::{
        PointerButton,
        Response,
        Sense,
        TextEdit,
        TextStyle,
        Ui,
        Widget,
    },
    emath::lerp,
    epaint::Pos2,
};
use egui::{
    vec2,
    Align2,
    Color32,
    Vec2,
};

use crate::{
    container::SlotState,
    widget_description::{
        visuals::{
            WidgetVisual,
            WidgetVisualKind,
            WidgetVisualMode,
        },
        KnobKind,
        WidFull,
        WidgetDescription,
        WidgetKind,
    },
};

use super::{
    KnobRange,
    SlotWidget,
};

fn calculate_value(value_range: KnobRange, angle: f32, angle_range: KnobRange) -> f32 {
    let normalized_angle = (angle - angle_range.start) / (angle_range.end - angle_range.start);
    lerp(value_range.start..=value_range.end, normalized_angle)
}

pub struct Knob {
    pos: Pos2,
    id: WidFull,

    value: f32,
    value_range: KnobRange,

    angle: f32,
    angle_range: KnobRange,

    default_angle: f32,

    speed: f32,
    size: Vec2,

    visuals: Vec<WidgetVisual>,
}

impl Knob {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pos: Pos2,
        id: WidFull,
        value_range: (f32, f32),
        angle_range: (f32, f32),
        default_pos: f32,
        speed: f32,
        size: Vec2,
        visuals: Vec<WidgetVisual>,
    ) -> Self {
        let angle_range = KnobRange::from((angle_range.0.to_radians(), angle_range.1.to_radians()));
        let value_range = KnobRange::from(value_range);
        let angle = lerp(angle_range.into(), default_pos);
        Self {
            pos,
            id,
            value: calculate_value(value_range, angle, angle_range),
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

    fn update(&mut self, ui: &mut Ui) -> Response {
        let old_value = self.value;
        let old_angle = self.angle;

        let mut res = self.allocate_space(ui);
        ui.painter()
            .debug_rect(res.rect, Color32::from_rgb(0, 255, 255), "");

        if res.clicked_by(PointerButton::Secondary) {
            self.angle = lerp(self.angle_range.into(), self.default_angle);
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

        self.draw(ui, &res, self.angle);

        res.changed = self.value != old_value;

        res
    }

    fn draw(&mut self, ui: &mut Ui, res: &Response, angle: f32) {
        let rect = res.rect;
        let center = rect.center();

        if ui.is_rect_visible(rect) {
            for visual in &self.visuals {
                match (&visual.mode, &visual.kind) {
                    (WidgetVisualMode::StateRelative, WidgetVisualKind::Line(end)) => {
                        let start = visual.center.to_vec2();
                        let (mut a, mut b) = (center.to_vec2(), center.to_vec2());
                        let dist = vec2(0., 0.) - start;
                        let len = vec2(0., 0.) - end.to_vec2();
                        a.x += dist.length() * angle.sin();
                        a.y += dist.length() * angle.cos();
                        b.x += len.length() * angle.sin();
                        b.y += len.length() * angle.cos();

                        ui.painter().line_segment(
                            [a.to_pos2(), b.to_pos2()],
                            ui.visuals().widgets.active.fg_stroke,
                        );
                    }
                    (_, WidgetVisualKind::Readout(size)) => {
                        let font = egui::FontId {
                            size: *size,
                            ..Default::default()
                        };
                        ui.painter().text(
                            center,
                            Align2::CENTER_CENTER,
                            format!("{}", self.value),
                            font,
                            ui.visuals().widgets.active.fg_stroke.color,
                        );
                    }
                    _ => {
                        visual.show(ui, center, Sense::hover());
                    }
                }
            }
        }
    }
}

impl Widget for &mut Knob {
    fn ui(self, ui: &mut Ui) -> Response {
        let ir = ui.vertical(|ui| self.update(ui));

        ir.inner | ir.response
    }
}

impl SlotWidget for Knob {
    fn pos(&self) -> Pos2 {
        self.pos
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn ui(&mut self, ui: &mut Ui, _extra_state: &mut SlotState) -> Response {
        <&mut Self as Widget>::ui(self, ui)
    }

    fn from_description(id: WidFull, description: WidgetDescription) -> Option<Self>
    where
        Self: Sized,
    {
        let WidgetDescription {
            kind:
                WidgetKind::Knob(KnobKind {
                    value_range,
                    angle_range,
                    default_pos,
                    speed,
                }),
            name: _,
            pos,
            size,
            visuals,
            extra: _,
        } = description
        else {
            return None;
        };

        let visuals = visuals.into_values().collect();

        Some(Self::new(
            pos,
            id,
            value_range,
            angle_range,
            default_pos,
            speed,
            size,
            visuals,
        ))
    }
}
