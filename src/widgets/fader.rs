use eframe::{
    egui::{PointerButton, Response, Sense, TextEdit, TextStyle, Ui, Widget},
    emath::lerp,
    epaint::{self, pos2, vec2, Pos2},
};
use std::{f32::consts::TAU, ops::RangeInclusive};

use super::KnobRange;

pub struct SimpleFader {
    pub value: f32,
    pub value_range: KnobRange,

    speed: f32,
}

impl SimpleFader {
    pub fn new(starting_pos: f32, value_range: (f32, f32), speed: f32) -> Self {
        Self {
            value: (),
            value_range: value_range.into(),
            speed: (),
        }
    }

    fn allocate_space(&self, ui: &mut Ui) -> Response {
        let size = vec2((self.radius + 1.0) * 2.0, (self.radius + 1.0) * 2.0);
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
            let mut ang = 0.0f32;
            let notch_angles = std::iter::repeat_with(|| {
                let a = TAU - ang.to_radians();
                ang += 10.0;
                a
            })
            .enumerate()
            .map(|(i, a)| (i % 9 == 0, a))
            .take(36);
            for (longer, angle) in notch_angles {
                let notch = {
                    let Pos2 { x, y } = rect.center();
                    let length = if longer { 6.0 } else { 4.0 };
                    [
                        pos2(
                            x + (self.radius - length) * angle.sin(),
                            y + (self.radius - length) * angle.cos(),
                        ),
                        pos2(
                            x + (self.radius - 1.0) * angle.sin(),
                            y + (self.radius - 1.0) * angle.cos(),
                        ),
                    ]
                };
                ui.painter().add(epaint::Shape::LineSegment {
                    points: notch,
                    stroke: ui.visuals().widgets.inactive.fg_stroke,
                });
            }
        }
    }
}

impl Widget for &mut SimpleFader {
    fn ui(self, ui: &mut Ui) -> Response {
        let ir = ui.vertical(|ui| self.update(ui));

        ir.inner | ir.response
    }
}
