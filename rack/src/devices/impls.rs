use self::{
    filters::MoogFilter,
    generators::{
        Osc,
        SquarePulse,
    },
    mixers::Attenuator,
};

use super::Device;

mod filters;
mod generators;
mod mixers;

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
