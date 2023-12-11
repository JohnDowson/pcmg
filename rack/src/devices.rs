use self::{
    description::{
        DeviceDescription,
        Param,
    },
    impls::{
        adsr::Adsr,
        filters::MoogFilter,
        generators::{
            Osc,
            SquarePulse,
        },
        mixers::{
            AbMixer,
            Attenuator,
        },
        sequencer::Sequencer,
    },
};

pub mod description;

pub mod impls;

pub trait Device {
    fn get_output_indexed(&mut self, idx: u8) -> f32;
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
    dd!(
        "A/B Mixer",
        [In("A"), In("B"), In("Ratio"), Out("Signal")],
        AbMixer::new()
    ),
    dd!(
        "ADSR",
        [
            In("A"),
            In("D"),
            In("S"),
            In("R"),
            In("A Ratio"),
            In("DR Ratio"),
            In("Trigger"),
            Out("Multipier")
        ],
        Adsr::new_simple(44000.)
    ),
    dd!(
        "Sequencer",
        [
            In("0"),
            In("1"),
            In("2"),
            In("3"),
            In("4"),
            In("5"),
            In("6"),
            In("7"),
            In("spb"),
            Out("Signal")
        ],
        Sequencer::new(44000.)
    ),
];

const CONTROL_PARAMS: &[Param] = &[Param::In("Control"), Param::Out("Output")];
const MIDI_PARAMS: &[Param] = &[Param::Out("Note"), Param::Out("Trigger")];
const OUTPUT_PARAMS: &[Param] = &[Param::In("Signal")];
