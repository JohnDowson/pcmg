use fusebox::FuseBox;
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    impls::{
        Control,
        MidiControl,
        Output,
    },
    Device,
    CONTROL_PARAMS,
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

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeviceKind {
    Control,
    MidiControl,
    #[serde(deserialize_with = "crate::de_device_description")]
    #[serde(serialize_with = "crate::ser_device_description")]
    Audio(usize),
    Output,
}

impl std::fmt::Debug for DeviceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
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

    pub fn num_params(&self) -> usize {
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
            DeviceKind::Control => CONTROL_PARAMS,
            DeviceKind::MidiControl => MIDI_PARAMS,
            DeviceKind::Audio(dd) => DEVICES[*dd].params,
            DeviceKind::Output => OUTPUT_PARAMS,
        }
    }

    pub fn make(&self) -> fn(&mut FuseBox<dyn Device + Send + Sync>) -> usize {
        match self {
            DeviceKind::Control => |d| {
                let i = d.len();
                d.push(Control(0.0));
                i
            },
            DeviceKind::MidiControl => |d| {
                let i = d.len();
                d.push(MidiControl(0.0, 0.0));
                i
            },

            DeviceKind::Audio(dd) => DEVICES[*dd].make,
            DeviceKind::Output => |d| {
                let i = d.len();
                d.push(Output(0.0));
                i
            },
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
