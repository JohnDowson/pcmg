use crate::types::{FilSel, GenSel, LfoSel, PipelineSelector};
use eframe::{
    egui::{PointerButton, Response, Sense, TextEdit, TextStyle, Ui, Widget},
    emath::{lerp, Real},
    epaint::{self, pos2, vec2, Pos2},
};
use num_traits::Float;
use std::{fmt::Display, ops::RangeInclusive};

pub mod fader;
pub mod knob;
pub mod scope;

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

type Transformer<V, T> = Box<dyn Fn(V) -> T + 'static>;

pub struct KnobBuilder<V, T> {
    value_range: Option<RangeInclusive<V>>,
    default_angle: f32,
    angle_range: RangeInclusive<f32>,

    transformer: Option<Transformer<V, T>>,

    label: Option<String>,
    speed: f32,
    radius: f32,
}

impl<V: Float + Real + 'static, T> KnobBuilder<V, T> {
    pub fn value_range(mut self, value_range: RangeInclusive<V>) -> Self {
        if value_range.start().is_sign_negative() && value_range.end().is_sign_positive() {
            self.default_angle = lerp(self.angle_range.clone(), 0.5);
        }
        self.value_range = Some(value_range);
        self
    }

    pub fn default_angle(mut self, default_angle: f32) -> Self {
        self.default_angle = default_angle;
        self
    }

    pub fn angle_range(mut self, angle_range: RangeInclusive<f32>) -> Self {
        if let Some(value_range) = &self.value_range {
            if value_range.start().is_sign_negative() && value_range.end().is_sign_positive() {
                self.default_angle = lerp(self.angle_range.clone(), 0.5);
            }
        }
        self.angle_range = angle_range;
        self
    }

    pub fn transformer(mut self, transformer: impl Fn(V) -> T + 'static) -> Self {
        self.transformer = Some(Box::new(transformer));
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn build(self) -> Knob<V, T> {
        let Self {
            value_range: Some(value_range),
            default_angle,
            angle_range,
            transformer: Some(transformer),
            label: Some(label),
            speed,
            radius,
        } = self
        else {
            panic!("Not all fields set")
        };

        Knob {
            value: calculate_value(value_range.clone(), default_angle, angle_range.clone()),
            value_range,
            default_angle,
            angle: default_angle,
            angle_range,

            transformer,
            label,
            speed,
            radius,
        }
    }
}

pub fn osc_group(
    i: usize,
    params: Vec<(GenSel, RangeInclusive<f32>)>,
) -> Vec<Knob<f32, PipelineSelector>> {
    params
        .into_iter()
        .map(|(param, range)| osc_knob(i, param, range))
        .chain(std::iter::once(level_knob(i)))
        .collect()
}

pub fn filter_group(
    i: usize,
    params: Vec<(FilSel, RangeInclusive<f32>)>,
) -> Vec<Knob<f32, PipelineSelector>> {
    params
        .into_iter()
        .map(|(param, range)| filter_knob(i, param, range))
        .collect()
}

pub fn lfo_knob(s: LfoSel, range: RangeInclusive<f32>) -> Knob<f32, PipelineSelector> {
    Knob::build()
        .label(format!("LFO {s:?}"))
        .value_range(range)
        .transformer(move |v| PipelineSelector::Lfo(s, v))
        .build()
}

pub fn level_knob(i: usize) -> Knob<f32, PipelineSelector> {
    Knob::build()
        .label(format!("Level {i}"))
        .value_range(0.0..=2.0)
        .transformer(move |v| PipelineSelector::Mixer(i, v))
        .build()
}

pub fn master_knob() -> Knob<f32, PipelineSelector> {
    Knob::build()
        .label("Master".to_string())
        .value_range(0.0..=2.0)
        .transformer(PipelineSelector::Master)
        .build()
}

pub fn osc_knob(i: usize, s: GenSel, range: RangeInclusive<f32>) -> Knob<f32, PipelineSelector> {
    Knob::build()
        .label(format!("OSC {i} {s:?}"))
        .value_range(range)
        .transformer(move |v| PipelineSelector::Osc(Some(i), s, v))
        .build()
}

pub fn filter_knob(i: usize, s: FilSel, range: RangeInclusive<f32>) -> Knob<f32, PipelineSelector> {
    Knob::build()
        .label(format!("Filter {i} {s:?}"))
        .value_range(range)
        .transformer(move |v| PipelineSelector::Filter(i, s, v))
        .build()
}

pub struct KnobGroup<V, T> {
    knobs: Vec<Vec<Knob<V, T>>>,
    changes: Vec<(usize, usize)>,
}

impl<V: Float + Real + Display + 'static, T> KnobGroup<V, T> {
    pub fn new(knobs: Vec<Vec<Knob<V, T>>>) -> Self {
        Self {
            knobs,
            changes: Vec::new(),
        }
    }
    pub fn changes(&'_ self) -> impl Iterator<Item = T> + '_ {
        self.changes
            .iter()
            .map(|&(y, x)| self.knobs[y][x].get_message())
    }
}

impl<V: Float + Real + Display + 'static, T> Widget for &mut KnobGroup<V, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.changes.clear();
        ui.horizontal(|ui| {
            for (y, group) in self.knobs.iter_mut().enumerate() {
                ui.vertical(|ui| {
                    for (x, knob) in group.iter_mut().enumerate() {
                        if ui.add(knob).changed() {
                            self.changes.push((y, x));
                        }
                    }
                });
            }
        })
        .response
    }
}

pub struct Knob<V, T> {
    value: V,
    value_range: RangeInclusive<V>,
    default_angle: f32,
    angle: f32,
    angle_range: RangeInclusive<f32>,

    transformer: Transformer<V, T>,
    label: String,
    speed: f32,
    radius: f32,
}

impl<V: Float + Real + Display, T> Knob<V, T> {
    pub fn build() -> KnobBuilder<V, T> {
        KnobBuilder {
            value_range: None,
            default_angle: 0.0,
            angle_range: 0.0f32.to_radians()..=360.0f32.to_radians(),
            transformer: None,
            label: None,
            speed: 0.1,
            radius: 16.0,
        }
    }

    pub fn get_message(&self) -> T {
        (self.transformer)(self.value)
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
                    (old_angle + delta).clamp(*self.angle_range.start(), *self.angle_range.end())
                } else {
                    old_angle
                }
            } else {
                old_angle
            };

            self.value = calculate_value(self.value_range.clone(), angle, self.angle_range.clone());
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
        ui.label(&self.label);
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
                let angle = std::f32::consts::TAU - (angle - 180.0.to_radians());
                pos2(x + self.radius * angle.sin(), y + self.radius * angle.cos())
            };
            ui.painter().add(epaint::Shape::LineSegment {
                points: [rect.center(), edge],
                stroke: ui.visuals().widgets.inactive.fg_stroke,
            });
        }
    }
}

fn calculate_value<T: Float + Real>(
    value_range: RangeInclusive<T>,
    angle: f32,
    angle_range: RangeInclusive<f32>,
) -> T {
    let normalized_angle =
        (angle - *angle_range.start()) / (*angle_range.end() - *angle_range.start());
    lerp(value_range, T::from(normalized_angle).unwrap())
}

impl<V: Float + Real + Display, T> Widget for &mut Knob<V, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let ir = ui.vertical(|ui| self.update(ui));

        ir.inner | ir.response
    }
}