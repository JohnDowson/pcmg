use std::collections::BTreeMap;

use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    container::sizing::ModuleSize,
    devices::description::DeviceKind,
    visuals::templates::WidgetTemplate,
    widgets::KnobRange,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleDescription {
    pub size: ModuleSize,
    pub visuals: BTreeMap<usize, WidgetTemplate>,
    pub devices: BTreeMap<usize, DeviceKind>,
    pub connections: BTreeMap<(usize, usize), usize>,
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

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum WidgetKind {
    Knob(KnobKind),
    #[default]
    Port,
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
    pub fn all() -> [Self; 2] {
        use WidgetKind::*;
        [Knob(Default::default()), Port]
    }
}
