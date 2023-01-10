use num_traits::{Float, FloatConst, FromPrimitive, Zero};
use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, SeedableRng};
use std::ops::Div;

pub trait Waveform<T>
where
    T: Float,
{
    fn oscilate(&mut self, phase: T) -> T;
}

impl<T, F> Waveform<T> for F
where
    T: Float,
    F: FnMut(T) -> T,
{
    fn oscilate(&mut self, phase: T) -> T {
        self(phase)
    }
}

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
        self.b[2] = 0.96900 * self.b[2] + s * 0.1538520;
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

pub struct Osc<T>
where
    T: Float,
{
    sample_rate: T,
    freq: T,
    d: T,
    phase: T,
    waveform: Box<dyn Waveform<T>>,
}

impl<T> Osc<T>
where
    T: Float + Zero + FromPrimitive + Div + FloatConst,
{
    pub fn new(sample_rate: T, waveform: Box<dyn Waveform<T>>) -> Self {
        let freq = T::zero();
        let d = freq / sample_rate;
        Self {
            sample_rate,
            freq,
            d,
            phase: T::zero(),
            waveform,
        }
    }

    pub fn with_freq(sample_rate: T, waveform: Box<dyn Waveform<T>>, freq: T) -> Self {
        let d = freq / sample_rate;
        Self {
            sample_rate,
            freq,
            d,
            phase: T::zero(),
            waveform,
        }
    }

    pub fn set_freq(&mut self, freq: T) {
        self.d = freq / self.sample_rate;
        self.freq = freq;
    }

    pub fn sample(&mut self) -> T {
        self.phase = (self.phase + self.d).fract();
        self.waveform.oscilate(self.phase * T::TAU())
    }

    pub fn modulate(&mut self, lfo: &mut Self) {
        let m = lfo.sample();
        let freq = self.freq;
        self.set_freq(freq + m)
    }
}
