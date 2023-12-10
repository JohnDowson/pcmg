use self::{
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
};

use super::Device;

pub mod adsr;
pub mod filters;
pub mod generators;
pub mod mixers;
pub mod sequencer;

pub struct Control(pub f32);

impl Device for Control {
    fn get_output_indexed(&mut self, _idx: u8) -> f32 {
        self.0
    }

    fn set_param_indexed(&mut self, _idx: u8, val: f32) {
        self.0 = val;
    }
}

pub struct Output(pub f32);

impl Device for Output {
    fn get_output_indexed(&mut self, _idx: u8) -> f32 {
        self.0
    }

    fn set_param_indexed(&mut self, _idx: u8, val: f32) {
        self.0 = val
    }
}

impl Device for SquarePulse<f32> {
    fn get_output_indexed(&mut self, _idx: u8) -> f32 {
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
    fn get_output_indexed(&mut self, _idx: u8) -> f32 {
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
    fn get_output_indexed(&mut self, _idx: u8) -> f32 {
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
    fn get_output_indexed(&mut self, _idx: u8) -> f32 {
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

impl Device for AbMixer {
    fn get_output_indexed(&mut self, _idx: u8) -> f32 {
        self.get_output()
    }

    fn set_param_indexed(&mut self, idx: u8, val: f32) {
        match idx {
            0 => self.set_a(val),
            1 => self.set_b(val),
            2 => self.set_ratio(val),
            _ => (),
        }
    }
}

impl Device for Sequencer {
    fn get_output_indexed(&mut self, _idx: u8) -> f32 {
        self.get_output()
    }

    fn set_param_indexed(&mut self, idx: u8, val: f32) {
        match idx {
            0..=7 => self.sequence[idx as usize] = val,
            8 => self.set_spb(val),
            _ => (),
        }
    }
}

impl Device for Adsr<f32> {
    fn get_output_indexed(&mut self, _idx: u8) -> f32 {
        self.apply()
    }

    fn set_param_indexed(&mut self, idx: u8, val: f32) {
        match idx {
            0 => self.set_attack_rate(val),
            1 => self.set_decay_rate(val),
            2 => self.set_sustain_level(val),
            3 => self.set_release_rate(val),
            4 => self.set_target_ratio_a(val),
            5 => self.set_target_ratio_dr(val),
            6 => self.set_input(val),
            _ => (),
        }
    }
}
