use std::collections::BTreeMap;

use serde::{
    Deserialize,
    Serialize,
};
use uuid::Uuid;

use crate::{
    container::sizing::ModuleSize,
    devices::description::DeviceKind,
    visuals::{
        templates::WidgetTemplate,
        VisualTheme,
    },
    widgets::KnobRange,
    Uuidentified,
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModuleConnectionSource {
    Device(usize, usize),
    Widget(usize),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleDescription {
    pub uuid: Uuid,
    pub name: String,
    pub theme: VisualTheme,
    pub size: ModuleSize,
    pub visuals: BTreeMap<usize, WidgetTemplate>,
    pub devices: BTreeMap<usize, DeviceKind>,
    pub connections: BTreeMap<ModuleConnectionSource, (usize, usize)>,
}

impl Uuidentified for ModuleDescription {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct KnobKind {
    pub angle_range: KnobRange,
    pub value_range: KnobRange,
    pub speed: f32,
}

impl Default for KnobKind {
    fn default() -> Self {
        Self {
            angle_range: (0., 360.).into(),
            value_range: (0., 1.).into(),
            speed: 0.1,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct ToggleKind {
    pub on: f32,
    pub off: f32,
}

impl Default for ToggleKind {
    fn default() -> Self {
        Self { on: 1., off: 0. }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum WidgetKind {
    Knob(KnobKind),
    Toggle(ToggleKind),
    #[default]
    Port,
}

impl std::fmt::Display for WidgetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WidgetKind::Knob(_) => f.write_str("Knob"),
            WidgetKind::Toggle(_) => f.write_str("Toggle"),
            _ => std::fmt::Debug::fmt(self, f),
        }
    }
}

impl WidgetKind {
    pub fn all() -> [Self; 3] {
        use WidgetKind::*;
        [Knob(Default::default()), Toggle(Default::default()), Port]
    }
}
