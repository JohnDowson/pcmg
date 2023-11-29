use self::{
    filters::MoogFilter,
    mixers::Attenuator,
};
use crate::types::{
    Osc,
    SquarePulse,
};
use fusebox::FuseBox;
use std::ops::RangeInclusive;

mod filters;
mod mixers;

macro_rules! __param_ty {
    (bool) => {
        ParamKind::Bool
    };
    (($start:literal ..= $end:literal)) => {
        ParamKind::Float(ParamRange($start, $end))
    };
}

macro_rules! dd {
    ($name:literal, $make:expr, $([$($param:literal : $param_ty:tt,)+])?) => {
        DeviceDescription {
            name: $name,
            params: &[
                $($(ParamDescription {
                    name: $param,
                    kind:__param_ty!($param_ty)
                },)+)?
            ],
            make: |fb| {
                let i = fb.len();
                #[allow(clippy::redundant_closure_call)]
                fb.push($make());
                i }
        }
    };
}

pub struct DeviceDescription {
    pub name: &'static str,
    pub params: &'static [ParamDescription],
    pub make: fn(&mut FuseBox<dyn Device + Send + Sync + 'static>) -> usize,
}

pub struct ParamDescription {
    pub name: &'static str,
    pub kind: ParamKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamKind {
    Bool,
    Float(ParamRange),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamRange(f32, f32);

impl From<ParamRange> for RangeInclusive<f32> {
    fn from(val: ParamRange) -> Self {
        val.0..=val.1
    }
}

impl From<RangeInclusive<f32>> for ParamRange {
    fn from(value: RangeInclusive<f32>) -> Self {
        Self(*value.start(), *value.end())
    }
}

pub static SYNTH_DESCRIPTIONS: &[DeviceDescription] = &[
    dd!("Square", || SquarePulse::<f32>::new(44000.0),
          [
            "Freq": (0.0..=22000.0),
            "Width": (0.0..=1.0),
            "Detune": (0.0..=180.0),
          ]
    ),
    dd!("Sine", || Osc::<f32>::new(44000.0, |p: f32| p.sin()),
          [
            "Freq": (0.0..=22000.0),
            "Detune": (0.0..=180.0),
          ]
    ),
];

pub static FILTER_DESCRIPTIONS: &[DeviceDescription] = &[
    dd!("Moog Filter", || MoogFilter::new(44000.0, 12000.0, 0.0),
    [
        "Input": (-1.0..=1.0),
        "Cutoff": (0.0..=22000.0),
        "Resonance": (0.0..=10.0),
    ]),
];

pub static MIXER_DESCRIPTIONS: &[DeviceDescription] =
    &[dd!("Attenuator", Attenuator::new, ["Input": (-1.0..=1.0),"Factor": (-10.0..=10.0),])];

pub trait Device {
    fn output(&mut self) -> f32;
    fn set_param_indexed(&mut self, idx: u8, val: f32);
}

pub struct Output(pub f32);

impl Device for Output {
    fn output(&mut self) -> f32 {
        self.0
    }

    fn set_param_indexed(&mut self, idx: u8, val: f32) {
        if 0 == idx {
            self.0 = val
        }
    }
}

impl Device for SquarePulse<f32> {
    fn output(&mut self) -> f32 {
        self.sample()
    }

    fn set_param_indexed(&mut self, idx: u8, val: f32) {
        match idx {
            0 => self.set_freq(val),
            1 => self.set_width(val),
            2 => self.set_detune(val),
            _ => (),
        }
    }
}

impl Device for Osc<f32> {
    fn output(&mut self) -> f32 {
        self.sample()
    }

    fn set_param_indexed(&mut self, idx: u8, val: f32) {
        match idx {
            0 => self.set_freq(val),
            1 => self.set_detune(val),
            _ => (),
        }
    }
}

impl Device for MoogFilter {
    fn output(&mut self) -> f32 {
        self.filter()
    }

    fn set_param_indexed(&mut self, idx: u8, val: f32) {
        match idx {
            0 => self.set_input(val),
            1 => self.set_cutoff(val),
            2 => self.set_resonance(val),
            _ => (),
        }
    }
}

impl Device for Attenuator {
    fn output(&mut self) -> f32 {
        self.get_output()
    }

    fn set_param_indexed(&mut self, idx: u8, val: f32) {
        match idx {
            0 => self.set_input(val),
            1 => self.set_factor(val),
            _ => (),
        }
    }
}
