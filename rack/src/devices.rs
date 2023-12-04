use self::{
    description::{
        DeviceDescription,
        Param,
    },
    impls::{
        filters::MoogFilter,
        generators::{
            Osc,
            SquarePulse,
        },
        mixers::Attenuator,
    },
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
pub(super) static DEVICES: &[DeviceDescription] = &[
    dd!(
        "SineOsc",
        [In("Freq"), In("Detune"), Out("Signal")],
        Osc::<f32>::new(44000.0, |p| p.sin())
    ),
    dd!(
        "SawOsc",
        [In("Freq"), In("Detune"), Out("Signal")],
        Osc::<f32>::new(44000.0, |p| p.sin().asin())
    ),
    dd!(
        "TriangleOsc",
        [In("Freq"), In("Detune"), Out("Signal")],
        Osc::<f32>::new(44000.0, |p| p.sin())
    ),
    dd!(
        "SquareOsc",
        [In("Freq"), In("Width"), Out("Signal")],
        SquarePulse::<f32>::new(44000.0)
    ),
    dd!(
        "MoogFilter",
        [In("Input"), In("Cutoff"), In("Resonance"), Out("Signal")],
        MoogFilter::new(44000.0, 1000.0, 0.0)
    ),
    dd!(
        "Attenuator",
        [In("Input"), In("Factor"), Out("Signal")],
        Attenuator::new()
    ),
];

const CONTROL_PARAMS: &[Param] = &[Param::Out("Control")];
const MIDI_PARAMS: &[Param] = &[Param::Out("Note")];
const OUTPUT_PARAMS: &[Param] = &[Param::In("Signal")];
