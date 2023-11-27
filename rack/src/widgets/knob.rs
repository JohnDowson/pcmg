use eframe::{
    egui::{PointerButton, Response, Sense, TextEdit, TextStyle, Ui, Widget},
    emath::lerp,
    epaint::{self, pos2, vec2, Pos2},
};
use egui::Vec2;
use std::f32::consts::TAU;

use crate::{
    container::{SlotState, StateValue},
    widget_description::{Sid, WidFull, WidgetDescription, WidgetKind},
};

use super::{KnobRange, SlotWidget};

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
    radius: f32,
}

impl Knob {
    pub fn new(
        pos: Pos2,
        id: WidFull,
        value_range: (f32, f32),
        angle_range: (f32, f32),
        default_pos: f32,
        speed: f32,
        radius: f32,
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
            radius,
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
        vec2((self.radius + 1.0) * 2.0, (self.radius + 1.0) * 2.0)
    }

    fn ui(&mut self, ui: &mut Ui, _extra_state: &mut SlotState) -> Response {
        <&mut Self as Widget>::ui(self, ui)
    }

    fn from_description(sid: Sid, description: &WidgetDescription) -> Option<Self>
    where
        Self: Sized,
    {
        let WidgetDescription {
            kind: WidgetKind::Knob,
            wid,
            name: _,
            pos,
            extra,
        } = description
        else {
            return None;
        };

        let StateValue::Range(value_range_start, value_range_end) = *extra.get("value_range")?
        else {
            return None;
        };
        let StateValue::Range(angle_range_start, angle_range_end) = *extra.get("angle_range")?
        else {
            return None;
        };
        let StateValue::Float(default_pos) = *extra.get("default_pos")? else {
            return None;
        };
        let StateValue::Float(speed) = *extra.get("speed")? else {
            return None;
        };
        let StateValue::Float(radius) = *extra.get("radius")? else {
            return None;
        };
        Some(Self::new(
            *pos,
            WidFull { sid, wid: *wid },
            (value_range_start, value_range_end),
            (angle_range_start, angle_range_end),
            default_pos,
            speed,
            radius,
        ))
    }
}
