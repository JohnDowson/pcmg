use std::ops::{
    AddAssign,
    Div,
};

use num::traits::{
    Float,
    FloatConst,
    FromPrimitive,
    Zero,
};

#[derive(Debug)]
pub struct SquarePulse<T>
where
    T: Float,
{
    sample_rate: T,
    freq: T,
    detune: T,
    width: T,
    d: T,
    phase: T,
}

impl<T> SquarePulse<T>
where
    T: Float + Zero + FromPrimitive + Div + FloatConst + AddAssign,
{
    pub fn new(sample_rate: T) -> Self {
        let freq = T::zero();
        let d = freq / sample_rate;
        Self {
            sample_rate,
            freq,
            detune: T::zero(),
            width: T::zero(),
            d,
            phase: T::zero(),
        }
    }

    pub fn with_params(sample_rate: T, freq: T, detune: T, duty: T) -> Self {
        let d = freq / sample_rate;
        Self {
            sample_rate,
            freq,
            detune,
            width: duty,
            d,
            phase: T::zero(),
        }
    }

    pub fn set_freq(&mut self, freq: T) {
        self.freq = freq;
        self.d = (self.freq + self.detune).max(T::zero()) / self.sample_rate;
    }

    pub fn set_width(&mut self, width: T) {
        self.width = width;
    }

    pub fn sample(&mut self) -> T {
        self.phase = (self.phase + self.d).fract();
        self.phase += (btf::<T>(self.phase >= T::one()) * -T::one())
            + (btf::<T>(self.phase < T::zero()) * T::one());
        (btf::<T>(self.phase < self.width) * T::one())
            + (btf::<T>(self.phase >= T::from_f32(0.5).unwrap())
                * btf::<T>(self.phase < T::from_f32(0.5).unwrap() + self.width)
                * -T::one())
    }

    pub fn set_detune(&mut self, detune: T) {
        self.detune = detune;
        self.set_freq(self.freq)
    }
}

fn btf<F: Float + FromPrimitive>(b: bool) -> F {
    F::from_u8(b as u8).unwrap()
}

pub struct FmOsc<T> {
    freq: T,
    carrier: Osc<T>,
    operators: [Osc<T>; 2],
    fm_ratios: [T; 2],
    fm_indexes: [T; 2],
}

impl<T: Float + FloatConst + FromPrimitive + Div + FloatConst + AddAssign> FmOsc<T> {
    pub fn new(sample_rate: T) -> Self {
        Self {
            freq: T::zero(),
            carrier: Osc::new(sample_rate, T::sin),
            operators: [Osc::new(sample_rate, T::sin), Osc::new(sample_rate, T::sin)],
            fm_ratios: [T::zero(); 2],
            fm_indexes: [T::zero(); 2],
        }
    }

    pub fn set_freq(&mut self, freq: T) {
        self.freq = freq;
    }

    pub fn set_fm_ratio(&mut self, n: usize, ratio: T) {
        self.fm_ratios[n] = ratio;
    }

    pub fn set_fm_index(&mut self, n: usize, index: T) {
        self.fm_indexes[n] = index;
    }

    pub fn sample(&mut self) -> T {
        let mut m = T::zero();
        for ((op, &ratio), &index) in self
            .operators
            .iter_mut()
            .zip(&self.fm_ratios)
            .zip(&self.fm_indexes)
        {
            op.set_freq(ratio + m);
            m = op.sample() * index;
        }
        let freq = self.freq + m;
        self.carrier.set_freq(freq);
        self.carrier.sample()
    }
}

pub struct Osc<T> {
    sample_rate: T,
    freq: T,
    detune: T,
    d: T,
    phase: T,
    waveform: fn(T) -> T,
}

impl<T> Osc<T>
where
    T: Float + Zero + FromPrimitive + Div + FloatConst + AddAssign,
{
    pub fn new(sample_rate: T, waveform: fn(T) -> T) -> Self {
        Self {
            sample_rate,
            freq: T::zero(),
            detune: T::zero(),
            d: T::zero(),
            phase: T::zero(),
            waveform,
        }
    }

    pub fn with_freq(sample_rate: T, waveform: fn(T) -> T, freq: T) -> Self {
        let mut this = Self::new(sample_rate, waveform);
        this.set_freq(freq);
        this
    }

    pub fn set_freq(&mut self, freq: T) {
        self.freq = freq;
        self.d = (self.freq + self.detune).max(T::zero()) / self.sample_rate;
    }

    pub fn sample(&mut self) -> T {
        self.phase = (self.phase + self.d).fract();
        (self.waveform)(self.phase * T::TAU())
    }

    pub fn set_detune(&mut self, detune: T) {
        self.detune = detune;
        self.set_freq(self.freq)
    }
}
