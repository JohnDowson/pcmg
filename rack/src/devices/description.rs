use fusebox::FuseBox;
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    Device,
    DEVICES,
    MIDI_PARAMS,
    OUTPUT_PARAMS,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceDescription {
    pub name: &'static str,
    pub params: &'static [Param],
    pub make: fn(&mut FuseBox<dyn Device + Send + Sync + 'static>) -> usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeviceKind {
    Control,
    MidiControl,
    #[serde(deserialize_with = "crate::de_device_description")]
    #[serde(serialize_with = "crate::ser_device_description")]
    Audio(usize),
    Output,
}

impl DeviceKind {
    pub fn all() -> Vec<DeviceKind> {
        let mut res = vec![
            DeviceKind::Control,
            DeviceKind::MidiControl,
            DeviceKind::Output,
        ];

        res.extend(
            DEVICES
                .iter()
                .enumerate()
                .map(|(i, _)| DeviceKind::Audio(i)),
        );

        res
    }

    pub fn num_values(&self) -> usize {
        self.params().len()
    }

    pub fn name(&self) -> &'static str {
        match self {
            DeviceKind::Control => "Control",
            DeviceKind::MidiControl => "MidiControl",
            DeviceKind::Audio(dd) => DEVICES[*dd].name,
            DeviceKind::Output => "Output",
        }
    }

    pub fn params(&self) -> &'static [Param] {
        match self {
            DeviceKind::Control => &[],
            DeviceKind::MidiControl => MIDI_PARAMS,
            DeviceKind::Audio(dd) => DEVICES[*dd].params,
            DeviceKind::Output => OUTPUT_PARAMS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Param {
    In(&'static str),
    Out(&'static str),
}

impl std::fmt::Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}
