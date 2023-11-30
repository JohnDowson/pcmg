use fusebox::FuseBox;

use super::{
    Device,
    DEVICES,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceDescription {
    pub name: &'static str,
    pub params: &'static [Param],
    pub make: fn(&mut FuseBox<dyn Device + Send + Sync + 'static>) -> usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceKind {
    Control,
    MidiControl,
    Audio(usize),
    Output,
}

impl DeviceKind {
    pub fn num_values(&self) -> usize {
        match self {
            DeviceKind::Control => todo!(),
            DeviceKind::MidiControl => todo!(),
            DeviceKind::Audio(dd) => DEVICES[*dd].params.len(),
            DeviceKind::Output => todo!(),
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
