use egui::{
    Pos2,
    Response,
    Ui,
    Vec2,
};
use std::ops::RangeInclusive;

use crate::{
    container::SlotState,
    widget_description::{
        WidFull,
        WidgetDescription,
    },
};

pub mod connector;
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

impl From<KnobRange> for RangeInclusive<f32> {
    fn from(v: KnobRange) -> Self {
        v.start..=v.end
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

pub trait SlotWidget {
    fn pos(&self) -> Pos2;
    fn size(&self) -> Vec2;
    fn value(&self) -> usize;
    fn show(&mut self, ui: &mut Ui, value: &mut f32, extra_state: &mut SlotState) -> Response;
    fn from_description(id: WidFull, description: WidgetDescription) -> Option<Self>
    where
        Self: Sized;
}
