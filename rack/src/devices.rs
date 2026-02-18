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
    ($name:literal, $params:expr, $make:expr) => {
        DeviceDescription {
            name: $name,
            params: &$params,
            make: |fb, sample_rate| {
                let i = fb.len();
                #[allow(clippy::redundant_closure_call)]
                fb.push(Box::new($make(sample_rate)));
                i
            },
        }
    };
}

use Param::*;
pub(super) static DEVICES: &[DeviceDescription] = &[
    dd!("SineOsc", [In("Freq"), In("Detune"), Out("Signal")], |sr| {
        Osc::<f32>::new(sr, |p| p.sin())
    }),
    dd!("SawOsc", [In("Freq"), In("Detune"), Out("Signal")], |sr| {
        Osc::<f32>::new(sr, |p| p.sin().asin())
    }),
    dd!(
        "TriangleOsc",
        [In("Freq"), In("Detune"), Out("Signal")],
        |sr| Osc::<f32>::new(sr, |p| p.sin())
    ),
    dd!(
        "SquareOsc",
        [In("Freq"), In("Width"), Out("Signal")],
        SquarePulse::<f32>::new
    ),
    dd!(
        "MoogFilter",
        [In("Input"), In("Cutoff"), In("Resonance"), Out("Signal")],
        |sr| MoogFilter::new(sr, 1000.0, 0.0)
    ),
    dd!(
        "Attenuator",
        [In("Input"), In("Factor"), Out("Signal")],
        |_| Attenuator::new()
    ),
    dd!(
        "A/B Mixer",
        [In("A"), In("B"), In("Ratio"), Out("Signal")],
        |_| AbMixer::new()
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
        Adsr::new_simple
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
            In("BPM"),
            Out("Signal")
        ],
        Sequencer::new
    ),
];

const CONTROL_PARAMS: &[Param] = &[Param::In("Control"), Param::Out("Output")];
const MIDI_PARAMS: &[Param] = &[Param::Out("Note"), Param::Out("Trigger")];
const OUTPUT_PARAMS: &[Param] = &[Param::In("Signal")];
