use num_traits::Float;

pub enum Stage {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct ADSR<T: num_traits::Float> {
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

impl<T: Float> ADSR<T> {
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
        if matches!(self.stage, Stage::Off) {
            self.stage = Stage::Attack;
        }
    }

    pub fn let_go(&mut self) {
        if !matches!(self.stage, Stage::Off) {
            self.stage = Stage::Release;
        }
    }

    pub fn apply(&mut self, sample: T) -> T {
        match self.stage {
            Stage::Off => T::zero(),
            Stage::Attack => {
                self.output = self.attack_base + self.output * self.attack_coef;
                if self.output >= T::one() {
                    self.output = T::one();
                    self.stage = Stage::Decay;
                }
                sample * self.output
            }
            Stage::Decay => {
                self.output = self.decay_base + self.output * self.decay_coef;
                if self.output <= self.sustain_level {
                    self.output = self.sustain_level;
                    self.stage = Stage::Sustain;
                }
                sample * self.output
            }
            Stage::Sustain => sample * self.sustain_level,
            Stage::Release => {
                self.output = self.release_base + self.output * self.release_coef;
                if self.output <= T::zero() {
                    self.output = T::zero();
                    self.stage = Stage::Off;
                }
                sample * self.output
            }
        }
    }

    pub fn set_attack_rate(&mut self, rate: T) {
        self.attack_rate = rate;
        self.attack_coef = Self::calculate_coef(rate, self.target_ratio_a);
        self.attack_base = (T::one() + self.target_ratio_a) * (T::one() - self.attack_coef);
    }

    pub fn set_decay_rate(&mut self, rate: T) {
        self.decay_rate = rate;
        self.decay_coef = Self::calculate_coef(rate, self.target_ratio_dr);
        self.decay_base =
            (self.sustain_level - self.target_ratio_dr) * (T::one() - self.decay_coef);
    }

    pub fn set_release_rate(&mut self, rate: T) {
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
        self.attack_base = (T::one() + self.target_ratio_a) * (T::one() - self.attack_coef);
    }

    pub fn set_target_ratio_dr(&mut self, mut ratio: T) {
        if ratio < T::from(0.000000001).unwrap() {
            ratio = T::from(0.000000001).unwrap(); // -180 dB
        }
        self.target_ratio_dr = ratio;
        self.decay_base =
            (self.sustain_level - self.target_ratio_dr) * (T::one() - self.decay_coef);
        self.release_base = -self.target_ratio_dr * (T::one() - self.release_coef);
    }

    fn calculate_coef(rate: T, ratio: T) -> T {
        (-(((T::one() + ratio) / ratio) / rate).log2()).exp()
    }
}

// impl<T: num_traits::Float + AddAssign + num_traits::ToPrimitive> ADSR<T> {
//     pub fn new(sample_rate: T, attack: T, decay: T, sustain: T, release: T) -> ADSR<T> {
//         ADSR {
//             stage: Stage::Off,
//             sample_rate,
//             curr_sample: 0,
//             next_stage_sample: 0,

//             last_sample: T::zero(),
//             multiplier: T::one(),

//             attack,
//             decay,
//             sustain,
//             release,
//         }
//     }

//     pub fn retrigger(&mut self) {
//         self.stage = Stage::Off;
//         self.advance_stage()
//     }

//     pub fn apply(&mut self, sample: T) -> T {
//         match self.stage {
//             Stage::Off => T::zero(),
//             Stage::Attack | Stage::Decay | Stage::Release => {
//                 if self.curr_sample == self.next_stage_sample {
//                     self.advance_stage();
//                 }
//                 self.curr_sample += 1;
//                 let sample = sample * self.multiplier;
//                 self.last_sample = sample;
//                 sample
//             }
//             Stage::Sustain => sample,
//         }
//     }

//     fn advance_stage(&mut self) {
//         self.curr_sample = 0;
//         self.stage = match self.stage {
//             Stage::Off => {
//                 self.next_stage_sample =
//                     (self.attack * self.sample_rate).round().to_usize().unwrap();
//                 self.calc_multiplier(T::zero(), T::one(), self.next_stage_sample);
//                 Stage::Attack
//             }
//             Stage::Attack => {
//                 self.next_stage_sample =
//                     (self.decay * self.sample_rate).round().to_usize().unwrap();
//                 self.calc_multiplier(T::zero(), T::one(), self.next_stage_sample);
//                 Stage::Decay
//             }
//             Stage::Decay => {
//                 self.next_stage_sample = 0;
//                 self.multiplier = T::one();
//                 Stage::Sustain
//             }
//             Stage::Sustain => {
//                 self.next_stage_sample = (self.release * self.sample_rate)
//                     .round()
//                     .to_usize()
//                     .unwrap();
//                 self.calc_multiplier(self.last_sample, T::zero(), self.next_stage_sample);
//                 Stage::Release
//             }
//             Stage::Release => {
//                 self.next_stage_sample = 0;
//                 Stage::Off
//             }
//         }
//     }

//     fn calc_multiplier(&mut self, start: T, end: T, length: usize) {
//         self.multiplier = T::one() + (end.log2() - start.log2()) / (T::from(length).unwrap());
//     }
// }
