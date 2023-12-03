use self::{
    description::{
        DeviceDescription,
        Param,
    },
    impls::generators::Osc,
};

pub mod description;

pub mod impls;

pub trait Device {
    fn output(&mut self) -> f32;
    fn set_param_indexed(&mut self, idx: u8, val: f32);
}

macro_rules! dd {
    ($name:literal, $params:expr, $($make:tt)+) => {
        DeviceDescription {
            name: $name,
            params: &$params,
            make: |fb| {
                let i = fb.len();
                fb.push($($make)+);
                i
            },
        }
    };
}

use Param::*;
pub static DEVICES: &[DeviceDescription] = &[dd!(
    "SineOsc",
    [In("Freq"), In("Detune"), Out("Signal")],
    Osc::<f32>::new(44000.0, |p| p.sin())
)];

pub const CONTROL_PARAMS: &[Param] = &[Param::Out("Control")];
pub const MIDI_PARAMS: &[Param] = &[Param::Out("Note")];
pub const OUTPUT_PARAMS: &[Param] = &[Param::In("Signal")];
