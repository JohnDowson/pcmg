use egui::{
    InnerResponse,
    Pos2,
    Ui,
    Vec2,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::ops::RangeInclusive;

use crate::{
    visuals::{
        templates::WidgetTemplate,
        VisualTheme,
    },
    Tooltipable,
};

use self::{
    connector::ports::Port,
    fader::Fader,
    knob::Knob,
};

pub mod connector;
pub mod fader;
pub mod knob;
pub mod scope;

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
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

pub enum WidgetResponse {
    None,
    Changed,
    AttemptConnection,
    AttemptDisconnect,
}

pub enum SlotWidget {
    Knob(Knob),
    Fader(Fader),
    Port(Port),
}

impl SlotWidget {
    pub fn pos(&self) -> Pos2 {
        match self {
            SlotWidget::Knob(k) => k.pos,
            SlotWidget::Fader(_f) => todo!(),
            SlotWidget::Port(p) => p.pos,
        }
    }

    pub fn size(&self) -> Vec2 {
        match self {
            SlotWidget::Knob(k) => k.size,
            SlotWidget::Fader(_f) => todo!(),
            SlotWidget::Port(p) => p.size,
        }
    }

    pub fn value(&self) -> f32 {
        match self {
            SlotWidget::Knob(k) => k.value,
            SlotWidget::Fader(_f) => todo!(),
            SlotWidget::Port(_) => 0.0,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, theme: VisualTheme) -> InnerResponse<WidgetResponse> {
        match self {
            SlotWidget::Knob(k) => k.show(ui, theme),
            SlotWidget::Fader(_f) => todo!(),
            SlotWidget::Port(p) => p.show(ui, theme),
        }
    }

    fn name(&self) -> String {
        match self {
            SlotWidget::Knob(k) => k.name.clone(),
            SlotWidget::Fader(_f) => todo!(),
            SlotWidget::Port(p) => p.name.clone(),
        }
    }
    pub fn from_template(template: WidgetTemplate) -> Option<Self> {
        template.into_slot_widget()
    }
}

impl Tooltipable for SlotWidget {
    fn tooltip(&self) -> String {
        self.name()
    }
}
