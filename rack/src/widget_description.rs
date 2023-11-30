use std::collections::BTreeMap;

use eframe::{
    egui::{
        Sense,
        Widget,
    },
    epaint::{
        Color32,
        Pos2,
        Rect,
    },
};
use egui::Vec2;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    container::{
        module::StateValue,
        sizing::ModuleSize,
    },
    devices::DeviceDescription,
    widgets::{
        connector::ports::Port,
        knob::Knob,
        SlotWidget,
    },
};

use visuals::WidgetVisual;

pub mod visuals;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ModuleDescription {
    pub size: ModuleSize,
    #[serde(serialize_with = "crate::ser_device_description")]
    #[serde(deserialize_with = "crate::de_device_description")]
    pub device: DeviceDescription,
    pub widgets: BTreeMap<u16, WidgetDescription>,
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct KnobKind {
    pub value_range: (f32, f32),
    pub angle_range: (f32, f32),
    pub default_pos: f32,
    pub speed: f32,
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum WidgetKind {
    #[default]
    Blinker,
    Knob(KnobKind),
    InPort,
    OutPort,
}

impl std::fmt::Display for WidgetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WidgetKind::Knob(_) => f.write_str("Knob"),
            _ => std::fmt::Debug::fmt(self, f),
        }
    }
}

impl WidgetKind {
    pub fn all() -> [Self; 4] {
        use WidgetKind::*;
        [Blinker, Knob(Default::default()), InPort, OutPort]
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WidgetDescription {
    pub kind: WidgetKind,
    pub name: String,
    #[serde(default)]
    pub value: usize,
    #[serde(default)]
    pub pos: Pos2,
    pub size: Vec2,
    #[serde(serialize_with = "crate::ser_btree_as_vec")]
    #[serde(deserialize_with = "crate::de_vec_as_btree")]
    pub visuals: BTreeMap<usize, WidgetVisual>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, StateValue>,
}

impl WidgetDescription {
    pub fn new(
        kind: WidgetKind,
        name: String,
        value: usize,
        pos: Pos2,
        size: Vec2,
        visuals: BTreeMap<usize, visuals::WidgetVisual>,
        extra: BTreeMap<String, StateValue>,
    ) -> Self {
        Self {
            kind,
            name,
            value,
            pos,
            size,
            visuals,
            extra,
        }
    }

    pub fn dyn_widget(self) -> Box<dyn SlotWidget> {
        match self.kind {
            WidgetKind::Blinker => {
                todo!()
            }
            WidgetKind::Knob(_) => Knob::from_description(self).map(Box::new).unwrap(),
            WidgetKind::OutPort | WidgetKind::InPort => {
                Port::from_description(self).map(Box::new).unwrap()
            }
        }
    }
}

impl Widget for &WidgetDescription {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let resp = ui.allocate_rect(
            Rect::from_min_size(self.pos, self.size),
            Sense::click_and_drag(),
        );

        for visual in self.visuals.values() {
            visual.show(ui, resp.rect.center(), Sense::hover());
        }
        ui.painter()
            .debug_rect(resp.rect, Color32::from_rgb(180, 170, 100), &self.name);

        resp
    }
}
