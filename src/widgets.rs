use std::{fmt::Display, ops::RangeInclusive};

use eframe::{
    egui::{CursorIcon, Response, Sense, TextEdit, TextStyle, Ui, Widget},
    emath::{lerp, Real},
    epaint::{self, pos2, vec2, Pos2},
};
use num_traits::Float;

type GetSet<'v, T> = Box<dyn 'v + FnMut(Option<(T, f32)>) -> (T, f32)>;

pub struct Knob<'v, T> {
    value: GetSet<'v, T>,
    value_range: RangeInclusive<T>,
    angle_range: RangeInclusive<f32>,

    radius: f32,
}

impl<'v, T: Float + Real + Display> Knob<'v, T> {
    pub fn new(
        value: &'v mut (T, f32),
        value_range: RangeInclusive<T>,
        angle_range: RangeInclusive<f32>,
    ) -> Self {
        let value: GetSet<'v, T> = Box::new(|v: Option<(T, f32)>| {
            if let Some(v) = v {
                *value = v;
            }
            *value
        });

        Self {
            value,
            value_range,
            angle_range,

            radius: 16.0,
        }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    fn allocate_space(&self, ui: &mut Ui) -> Response {
        let size = vec2(self.radius * 2.0, self.radius * 2.0);
        ui.allocate_response(size, Sense::drag())
    }

    fn update(&mut self, ui: &mut Ui) -> Response {
        let (old_value, old_angle) = (self.value)(None);

        let mut res = self.allocate_space(ui);
        self.draw(ui, &res, old_angle);

        let angle = if res.dragged() {
            ui.output().cursor_icon = CursorIcon::ResizeHorizontal;
            let delta = res.drag_delta();
            let delta = delta.x - delta.y;
            let delta = delta * 0.1;

            if delta != 0.0 {
                // Since we round the value being dragged, we need to store the full precision value in memory:

                (old_angle + delta).clamp(*self.angle_range.start(), *self.angle_range.end())
            } else {
                old_angle
            }
        } else {
            old_angle
        };
        let normalized_angle = (angle - *self.angle_range.start())
            / (*self.angle_range.end() - *self.angle_range.start());
        let new_value = lerp(self.value_range.clone(), T::from(normalized_angle).unwrap());
        let _ = (self.value)(Some((new_value, angle)));

        let mut text = format!("A:{angle} N:{normalized_angle} V:{new_value}");
        let text_res = ui.add(
            TextEdit::singleline(&mut text)
                .interactive(false)
                // .desired_width(self.radius * 2.0)
                .font(TextStyle::Monospace),
        );

        res |= text_res;
        res.changed = new_value != old_value;

        res
    }

    fn draw(&mut self, ui: &mut Ui, res: &Response, angle: f32) {
        let rect = res.rect;

        if ui.is_rect_visible(rect) {
            ui.painter().add(epaint::CircleShape {
                center: rect.center(),
                radius: self.radius,
                fill: ui.visuals().widgets.inactive.bg_fill,
                stroke: ui.visuals().widgets.inactive.bg_stroke,
            });
            let edge = {
                let Pos2 { x, y } = rect.center();
                pos2(x + self.radius * angle.sin(), y + self.radius * angle.cos())
            };
            ui.painter().add(epaint::Shape::LineSegment {
                points: [rect.center(), edge],
                stroke: ui.visuals().widgets.inactive.fg_stroke,
            });
        }
    }
}

impl<'v, T: Float + Real + Display> Widget for Knob<'v, T> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let ir = ui.vertical(|ui| self.update(ui));
        ir.inner | ir.response
    }
}
