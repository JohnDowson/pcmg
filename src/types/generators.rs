use num_traits::{
    Float,
    FloatConst,
    FromPrimitive,
    Zero,
};
use rand::{
    distributions::Uniform,
    prelude::Distribution,
    rngs::StdRng,
    SeedableRng,
};
use std::ops::{
    AddAssign,
    Div,
};

pub struct WhiteNoise {
    rng: StdRng,
    dist: Uniform<f32>,
}

impl WhiteNoise {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
            dist: Uniform::new(-1.0, 1.0),
        }
    }

    pub fn sample(&mut self) -> f32 {
        self.dist.sample(&mut self.rng)
    }
}

impl Default for WhiteNoise {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PinkNoise {
    source: WhiteNoise,
    b: [f32; 7],
}

impl PinkNoise {
    pub fn new() -> Self {
        Self {
            source: WhiteNoise::new(),
            b: [0.0; 7],
        }
    }

    pub fn sample(&mut self) -> f32 {
        let s = self.source.sample();

        self.b[0] = 0.99886 * self.b[0] + s * 0.0555179;
        self.b[1] = 0.99332 * self.b[1] + s * 0.0750759;
        self.b[2] = 0.96900 * self.b[2] + s * 0.153_852;
        self.b[3] = 0.86650 * self.b[3] + s * 0.3104856;
        self.b[4] = 0.55000 * self.b[4] + s * 0.5329522;
        self.b[5] = -0.7616 * self.b[5] - s * 0.0168980;
        let pink = (self.b[0]
            + self.b[1]
            + self.b[2]
            + self.b[3]
            + self.b[4]
            + self.b[5]
            + self.b[6]
            + (s * 0.5362))
            * 0.11;
        self.b[6] = s * 0.115926;
        pink
    }
}

impl Default for PinkNoise {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BrownNoise {
    source: WhiteNoise,
    l: f32,
}

impl BrownNoise {
    pub fn new() -> Self {
        Self {
            source: WhiteNoise::new(),
            l: 0.0,
        }
    }

    pub fn sample(&mut self) -> f32 {
        let s = self.source.sample();

        let brown = (self.l + (0.02 * s)) / 1.02;
        self.l = brown;
        brown * 3.5
    }
}

impl Default for BrownNoise {
    fn default() -> Self {
        Self::new()
    }
}

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
