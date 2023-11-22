use eframe::{
    egui::{PointerButton, Response, Sense, TextEdit, TextStyle, Ui, Widget},
    emath::lerp,
    epaint::{self, pos2, vec2, Pos2},
};
use std::{f32::consts::TAU, ops::RangeInclusive};

#[derive(Clone, Copy)]
pub struct KnobRange {
    pub start: f32,
    pub end: f32,
}

impl From<RangeInclusive<f32>> for KnobRange {
    fn from(range: RangeInclusive<f32>) -> Self {
        Self {
            start: *range.start(),
            end: *range.end(),
        }
    }
}

impl From<(f32, f32)> for KnobRange {
    fn from(range: (f32, f32)) -> Self {
        Self {
            start: range.0,
            end: range.1,
        }
    }
}

fn calculate_value(value_range: KnobRange, angle: f32, angle_range: KnobRange) -> f32 {
    let normalized_angle = (angle - angle_range.start) / (angle_range.end - angle_range.start);
    lerp(value_range.start..=value_range.end, normalized_angle)
}

pub struct SimpleKnob {
    pub value: f32,
    pub value_range: KnobRange,
    default_angle: f32,
    angle: f32,
    angle_range: KnobRange,

    speed: f32,
    radius: f32,
}

impl SimpleKnob {
    pub fn new(
        value_range: (f32, f32),
        angle_range: (f32, f32),
        default_angle: f32,
        speed: f32,
        radius: f32,
    ) -> Self {
        let angle_range = KnobRange::from((angle_range.0.to_radians(), angle_range.1.to_radians()));
        let value_range = KnobRange::from(value_range);
        let default_angle = default_angle.to_radians();
        Self {
            value: calculate_value(value_range, default_angle, angle_range),
            value_range,
            default_angle,
            angle: default_angle,
            angle_range,
            speed,
            radius,
        }
    }

    fn allocate_space(&self, ui: &mut Ui) -> Response {
        let size = vec2(self.radius * 2.0, self.radius * 2.0);
        ui.allocate_response(size, Sense::click_and_drag())
    }

    fn update(&mut self, ui: &mut Ui) -> Response {
        let old_value = self.value;
        let old_angle = self.angle;

        let mut res = self.allocate_space(ui);

        self.draw(ui, &res, old_angle);

        if res.clicked_by(PointerButton::Secondary) {
            self.angle = self.default_angle;
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

        let mut text = self.value.to_string();
        let text_res = ui.add(
            TextEdit::singleline(&mut text)
                .interactive(false)
                .desired_width(self.radius * 2.0)
                .font(TextStyle::Monospace),
        );

        res |= text_res;
        res.changed = self.value != old_value;

        res
    }

    fn draw(&mut self, ui: &mut Ui, res: &Response, angle: f32) {
        let rect = res.rect;

        if ui.is_rect_visible(rect) {
            let stroke = if res.dragged() {
                ui.visuals().widgets.active.bg_stroke
            } else {
                ui.visuals().widgets.inactive.bg_stroke
            };
            ui.painter().add(epaint::CircleShape {
                center: rect.center(),
                radius: self.radius,
                fill: ui.visuals().widgets.inactive.bg_fill,
                stroke,
            });
            let edge = {
                let Pos2 { x, y } = rect.center();
                let angle = TAU - angle;
                pos2(x + self.radius * angle.sin(), y + self.radius * angle.cos())
            };
            ui.painter().add(epaint::Shape::LineSegment {
                points: [rect.center(), edge],
                stroke: ui.visuals().widgets.inactive.fg_stroke,
            });
        }
    }
}

impl Widget for &mut SimpleKnob {
    fn ui(self, ui: &mut Ui) -> Response {
        let ir = ui.vertical(|ui| self.update(ui));

        ir.inner | ir.response
    }
}
