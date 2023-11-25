use std::iter;

use eframe::{
    egui::{PointerButton, Response, Sense, TextEdit, TextStyle, Ui, Widget},
    emath::lerp,
    epaint::{self, pos2, vec2, Color32, Rounding},
};

use super::KnobRange;

fn calculate_value(value_range: KnobRange, position: f32) -> f32 {
    lerp(value_range.start..=value_range.end, position)
}

pub struct Fader {
    pub value: f32,
    pub value_range: KnobRange,
    pub pos: f32,
    starting_pos: f32,

    speed: f32,
}

impl Fader {
    pub fn new(starting_pos: f32, value_range: (f32, f32), speed: f32) -> Self {
        let value = calculate_value(value_range.into(), starting_pos);

        Self {
            value,
            value_range: value_range.into(),
            pos: starting_pos,
            starting_pos,
            speed,
        }
    }

    fn allocate_space(&self, ui: &mut Ui) -> Response {
        let size = vec2(16.0, 256.0);
        ui.allocate_response(size, Sense::click_and_drag())
    }

    fn update(&mut self, ui: &mut Ui) -> Response {
        let old_pos = self.pos;
        let old_value = self.value;

        let mut res = self.allocate_space(ui);

        self.draw(ui, &res, old_pos);

        if res.clicked_by(PointerButton::Secondary) {
            self.pos = self.starting_pos;
        } else {
            let pos = if res.dragged() {
                let delta = res.drag_delta();
                let delta = delta.y / 256.0;
                let delta = (-delta) * self.speed;

                if delta != 0.0 {
                    (old_pos + delta).clamp(0.0, 1.0)
                } else {
                    old_pos
                }
            } else {
                old_pos
            };

            self.value = calculate_value(self.value_range, pos);
            self.pos = pos;
        }

        let mut text = self.value.to_string();
        let text_res = ui.add(
            TextEdit::singleline(&mut text)
                .interactive(false)
                .desired_width(32.0)
                .font(TextStyle::Monospace),
        );

        res |= text_res;
        res.changed = self.value != old_value;

        res
    }

    fn draw(&mut self, ui: &mut Ui, res: &Response, pos: f32) {
        let rect = res.rect;

        if ui.is_rect_visible(rect) {
            let fg = ui.visuals().widgets.inactive.fg_stroke;
            let bg = ui.visuals().widgets.active.bg_stroke;

            let start = rect.center_top();
            let end = rect.center_bottom();
            ui.painter().add(epaint::Shape::LineSegment {
                points: [start, end],
                stroke: epaint::Stroke {
                    width: 2.0,
                    color: Color32::from_rgb(0, 0, 0),
                },
            });
            let mut nth = pos2(start.x - 4.0, start.y);
            let (mut ox, oy) = (8.0, 8.0);

            let segs = iter::repeat_with(|| {
                let n1 = nth;
                nth = pos2(nth.x + ox, nth.y + oy);
                ox = -ox;
                [n1, nth]
            })
            .take(32);

            for seg in segs {
                ui.painter().add(epaint::Shape::LineSegment {
                    points: seg,
                    stroke: bg,
                });
            }

            let mut handle_rect = rect.shrink2(vec2(0.0, 126.0));
            let center_x = rect.center_top().x;
            let center_y = lerp(
                (rect.center_bottom().y - 2.0)..=(rect.center_top().y + 2.0),
                pos,
            );
            handle_rect.set_center(pos2(center_x, center_y));
            ui.painter().add(epaint::RectShape::filled(
                handle_rect,
                Rounding::same(1.6),
                fg.color,
            ));
        }
    }
}

impl Widget for &mut Fader {
    fn ui(self, ui: &mut Ui) -> Response {
        let ir = ui.vertical(|ui| self.update(ui));

        ir.inner | ir.response
    }
}
