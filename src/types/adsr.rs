use num_traits::{
    Float,
    FloatConst,
};

// translated from
// http://www.earlevel.com/main/2013/06/03/envelope-generators-adsr-code/

#[derive(Debug)]
pub enum Stage {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug)]
pub struct Adsr<T: num_traits::Float> {
    stage: Stage,
    sample_rate: T,
    output: T,

    attack_rate: T,
    decay_rate: T,
    release_rate: T,
    attack_coef: T,
    decay_coef: T,
    release_coef: T,
    sustain_level: T,

    target_ratio_a: T,
    target_ratio_dr: T,
    attack_base: T,
    decay_base: T,
    release_base: T,
}

impl<T: Float + FloatConst> Adsr<T> {
    pub fn new(
        sample_rate: T,
        attack: T,
        decay: T,
        sustain: T,
        release: T,
        ratio_a: T,
        ratio_dr: T,
    ) -> Self {
        let mut this = Self {
            stage: Stage::Off,
            sample_rate,
            output: T::zero(),

            attack_rate: T::zero(),
            decay_rate: T::zero(),
            release_rate: T::zero(),
            attack_coef: T::zero(),
            decay_coef: T::zero(),
            release_coef: T::zero(),
            sustain_level: T::zero(),

            target_ratio_a: T::zero(),
            target_ratio_dr: T::zero(),
            attack_base: T::zero(),
            decay_base: T::zero(),
            release_base: T::zero(),
        };

        this.set_attack_rate(attack);
        this.set_decay_rate(decay);
        this.set_release_rate(release);
        this.set_sustain_level(sustain);
        this.set_target_ratio_a(ratio_a);
        this.set_target_ratio_dr(ratio_dr);

        this
    }

    pub fn trigger(&mut self) {
        self.stage = Stage::Attack;
    }

    pub fn hold(&mut self) {
        self.stage = Stage::Sustain;
    }

    pub fn let_go(&mut self) {
        if !matches!(self.stage, Stage::Off) {
            self.stage = Stage::Release;
        }
    }

    pub fn apply(&mut self, sample: T) -> T {
        match self.stage {
            Stage::Off => self.output = T::zero(),
            Stage::Attack => {
                self.output = self.attack_base + self.output * self.attack_coef;
                if self.output >= T::one() {
                    self.output = T::one();
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                self.output = self.decay_base + self.output * self.decay_coef;
                if self.output <= self.sustain_level {
                    self.output = self.sustain_level;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => self.output = self.sustain_level,
            Stage::Release => {
                self.output = self.release_base + self.output * self.release_coef;
                if self.output <= T::zero() {
                    self.output = T::zero();
                    self.stage = Stage::Off;
                }
            }
        }
        sample * self.output
    }

    pub fn get_output(&self) -> T {
        self.output
    }

    pub fn set_attack_rate(&mut self, rate: T) {
        let rate = rate * self.sample_rate;
        self.attack_rate = rate;
        self.attack_coef = Self::calculate_coef(rate, self.target_ratio_a);
        self.attack_base = (T::one() + self.target_ratio_a) * (T::one() - self.attack_coef);
    }

    pub fn set_decay_rate(&mut self, rate: T) {
        let rate = rate * self.sample_rate;
        self.decay_rate = rate;
        self.decay_coef = Self::calculate_coef(rate, self.target_ratio_dr);
        self.decay_base =
            (self.sustain_level - self.target_ratio_dr) * (T::one() - self.decay_coef);
    }

    pub fn set_release_rate(&mut self, rate: T) {
        let rate = rate * self.sample_rate;
        self.release_rate = rate;
        self.release_coef = Self::calculate_coef(rate, self.target_ratio_dr);
        self.release_base = -self.target_ratio_dr * (T::one() - self.release_coef)
    }

    pub fn set_sustain_level(&mut self, level: T) {
        self.sustain_level = level;
        self.decay_base = (T::one() + self.target_ratio_dr) * (T::one() - self.decay_coef);
    }

    pub fn set_target_ratio_a(&mut self, mut ratio: T) {
        if ratio < T::from(0.000000001).unwrap() {
            ratio = T::from(0.000000001).unwrap(); // -180 dB
        }
        self.target_ratio_a = ratio;
        self.attack_coef = Self::calculate_coef(self.attack_rate, self.target_ratio_a);
        self.attack_base = (T::one() + self.target_ratio_a) * (T::one() - self.attack_coef);
    }

    pub fn set_target_ratio_dr(&mut self, mut ratio: T) {
        if ratio < T::from(0.000000001).unwrap() {
            ratio = T::from(0.000000001).unwrap(); // -180 dB
        }
        self.target_ratio_dr = ratio;
        self.decay_coef = Self::calculate_coef(self.decay_rate, self.target_ratio_dr);
        self.release_coef = Self::calculate_coef(self.release_rate, self.target_ratio_dr);
        self.decay_base =
            (self.sustain_level - self.target_ratio_dr) * (T::one() - self.decay_coef);
        self.release_base = -self.target_ratio_dr * (T::one() - self.release_coef);
    }

    fn calculate_coef(rate: T, ratio: T) -> T {
        (-((T::one() + ratio) / ratio).log(T::E()) / rate).exp()
    }
}
