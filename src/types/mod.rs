#![allow(dead_code)]
mod adsr;
mod errors;
pub mod filters;
mod fused;
pub mod generators;
mod hz;
mod lfo;
mod note;

use std::ptr;

pub use adsr::*;
pub use errors::*;
pub use hz::*;
pub use lfo::*;
pub use note::*;

use self::{
    filters::{KrajeskiLadder, MoogFilter},
    fused::Fused,
    generators::{BrownNoise, Osc, PinkNoise, WhiteNoise},
};

// TODO
pub struct Instrument {
    envelope: ADSR<f64>,
    lfo: LFO<f64>,
}

pub struct Mixer<const N: usize> {
    levels: [f32; N],
}

impl<const N: usize> Mixer<N> {
    pub fn new(levels: [f32; N]) -> Self {
        Self { levels }
    }
    pub fn mix(&self, n: usize, s: f32) -> f32 {
        s * self.levels[n]
    }
    pub fn set_level(&mut self, n: usize, l: f32) {
        self.levels[n] = l;
    }
}

pub struct FusedGeneratorIterator<'g> {
    gen: &'g mut FusedGenerator,
    n: usize,
}

impl<'g> FusedGeneratorIterator<'g> {
    fn next(&mut self) -> Option<(usize, &mut dyn Generator<Selector = &'static str>)> {
        if self.n == self.gen.inner.len() {
            return None;
        }
        let r = (self.n, self.gen.get(self.n));
        self.n += 1;
        Some(r)
    }
}

pub struct FusedGenerator {
    inner: Fused<dyn Generator<Selector = &'static str>>,
}

impl FusedGenerator {
    pub fn new() -> Self {
        Self {
            inner: Fused::new(),
        }
    }

    pub fn iter(&mut self) -> FusedGeneratorIterator {
        FusedGeneratorIterator { gen: self, n: 0 }
    }

    pub fn set_param(&mut self, param: (usize, &'static str), val: f32) {
        self.get(param.which()).set_param(param.param(), val)
    }

    pub fn push<G: Generator<Selector = &'static str> + 'static>(&mut self, g: G) {
        let meta = ptr::metadata(&g as &dyn Generator<Selector = &'static str>);
        self.inner.push(g, meta)
    }

    pub fn get(&mut self, n: usize) -> &mut dyn Generator<Selector = &'static str> {
        self.inner.get_dyn(n)
    }
}

pub trait Generator {
    type Selector: Selector;
    fn sample(&mut self) -> f32;
    fn set_param(&mut self, param: Self::Selector, val: f32);
}

impl Generator for Osc<f32> {
    type Selector = &'static str;

    fn sample(&mut self) -> f32 {
        self.sample()
    }
    fn set_param(&mut self, param: Self::Selector, val: f32) {
        match param {
            "freq" => self.set_freq(val),
            _ => (),
        }
    }
}

impl Generator for WhiteNoise {
    type Selector = &'static str;

    fn sample(&mut self) -> f32 {
        self.sample()
    }

    fn set_param(&mut self, _param: Self::Selector, _val: f32) {}
}
impl Generator for PinkNoise {
    type Selector = &'static str;

    fn sample(&mut self) -> f32 {
        self.sample()
    }

    fn set_param(&mut self, _param: Self::Selector, _val: f32) {}
}
impl Generator for BrownNoise {
    type Selector = &'static str;

    fn sample(&mut self) -> f32 {
        self.sample()
    }

    fn set_param(&mut self, _param: Self::Selector, _val: f32) {}
}

pub trait Selector {
    fn param(&self) -> &'static str;
    fn which(&self) -> usize;
}

impl Selector for () {
    fn param(&self) -> &'static str {
        ""
    }

    fn which(&self) -> usize {
        0
    }
}

impl Selector for &'static str {
    fn param(&self) -> &'static str {
        *self
    }

    fn which(&self) -> usize {
        0
    }
}

impl Selector for (usize, &'static str) {
    fn param(&self) -> &'static str {
        self.1
    }

    fn which(&self) -> usize {
        self.0
    }
}

pub enum PipelineSelector<FS: Selector> {
    Osc((usize, &'static str), f32),
    Mixer(usize, f32),
    Filter(FS, f32),
}

pub trait Filter {
    type Selector: Selector;
    fn filter(&mut self, sample: f32) -> f32;
    fn set_param(&mut self, param: Self::Selector, val: f32);
}

impl Filter for KrajeskiLadder {
    type Selector = &'static str;

    fn filter(&mut self, sample: f32) -> f32 {
        self.filter(sample)
    }

    fn set_param(&mut self, param: Self::Selector, val: f32) {
        match param {
            "cutoff" => self.set_cutoff(val),
            "resonance" => self.set_resonance(val),
            _ => (),
        }
    }
}

impl Filter for MoogFilter {
    type Selector = &'static str;

    fn filter(&mut self, sample: f32) -> f32 {
        self.filter(sample)
    }

    fn set_param(&mut self, param: Self::Selector, val: f32) {
        match param {
            "cutoff" => self.set_cutoff(val),
            "resonance" => self.set_resonance(val),
            _ => (),
        }
    }
}

impl<S: Selector> Filter for Box<dyn Filter<Selector = S>> {
    type Selector = S;

    fn filter(&mut self, sample: f32) -> f32 {
        self.as_mut().filter(sample)
    }

    fn set_param(&mut self, param: Self::Selector, val: f32) {
        self.as_mut().set_param(param, val)
    }
}

impl<F: Filter<Selector = &'static str>, const N: usize> Filter for [F; N] {
    type Selector = (usize, &'static str);
    fn filter(&mut self, mut sample: f32) -> f32 {
        for f in self.iter_mut() {
            sample = f.filter(sample);
        }
        sample
    }

    fn set_param(&mut self, param: Self::Selector, val: f32) {
        self[param.which()].set_param(param.param(), val);
    }
}
pub struct Pipeline<F: Filter, const N: usize> {
    oscs: FusedGenerator,
    mixer: Mixer<N>,
    filters: F,
}

impl<F: Filter, const N: usize> Pipeline<F, N> {
    pub fn new(oscs: FusedGenerator, levels: [f32; N], filters: F) -> Self {
        let mixer = Mixer::new(levels);
        Self {
            oscs,
            mixer,
            filters,
        }
    }

    pub fn sample(&mut self) -> f32 {
        let mut oscs = self.oscs.iter();
        let mut sample = 0.0;
        while let Some((i, s)) = oscs.next() {
            sample += self.mixer.mix(i, s.sample())
        }
        self.filters.filter(sample)
    }

    pub fn set_param(&mut self, param: PipelineSelector<F::Selector>) {
        match param {
            PipelineSelector::Osc(p, v) => self.oscs.set_param(p, v),
            PipelineSelector::Mixer(n, l) => self.mixer.set_level(n, l),
            PipelineSelector::Filter(s, v) => self.filters.set_param(s, v),
        }
    }
}
