#![allow(dead_code)]
mod adsr;
mod errors;
pub mod filters;
mod fused;
pub mod generators;
mod hz;
mod note;

use self::{
    filters::{KrajeskiLadder, MoogFilter},
    fused::Fused,
    generators::{BrownNoise, Osc, PinkNoise, SquarePulse, WhiteNoise},
};
pub use adsr::*;
pub use errors::*;
pub use hz::*;
pub use note::*;
use std::ptr;

pub struct FusedGeneratorIterator<'g> {
    gen: &'g mut FusedGenerator,
    n: usize,
}

impl<'g> FusedGeneratorIterator<'g> {
    fn next(&mut self) -> Option<(usize, &mut dyn Generator)> {
        if self.n == self.gen.inner.len() {
            return None;
        }
        let r = (self.n, self.gen.get(self.n));
        self.n += 1;
        Some(r)
    }
}

pub struct FusedGenerator {
    inner: Fused<dyn Generator>,
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

    pub fn set_param(&mut self, n: usize, param: GenSel, val: f32) {
        self.get(n).set_param(param, val)
    }

    pub fn push<G: Generator + 'static>(&mut self, g: G) {
        let meta = ptr::metadata(&g as &dyn Generator);
        self.inner.push(g, meta)
    }

    pub fn get(&mut self, n: usize) -> &mut dyn Generator {
        self.inner.get_dyn(n)
    }
}

pub struct FusedFilter {
    inner: Fused<dyn Filter>,
}

impl FusedFilter {
    pub fn new() -> Self {
        Self {
            inner: Fused::new(),
        }
    }

    pub fn filter(&mut self, mut sample: f32) -> f32 {
        for i in 0..self.inner.len() {
            sample = self.get(i).filter(sample)
        }
        sample
    }

    pub fn set_param(&mut self, n: usize, param: FilSel, val: f32) {
        self.get(n).set_param(param, val)
    }

    pub fn push<G: Filter + 'static>(&mut self, g: G) {
        let meta = ptr::metadata(&g as &dyn Filter);
        self.inner.push(g, meta)
    }

    pub fn get(&mut self, n: usize) -> &mut dyn Filter {
        self.inner.get_dyn(n)
    }
}

pub trait Generator: Parametrise<Selector = GenSel> {
    fn sample(&mut self) -> f32;
}

pub trait Parametrise {
    type Selector;
    fn set_param(&mut self, param: Self::Selector, val: f32);
}

impl Parametrise for SquarePulse<f32> {
    type Selector = GenSel;

    fn set_param(&mut self, param: Self::Selector, val: f32) {
        match param {
            GenSel::Freq => self.set_freq(val),
            GenSel::Width => self.set_width(val),
        }
    }
}

impl Generator for SquarePulse<f32> {
    fn sample(&mut self) -> f32 {
        self.sample()
    }
}

impl Parametrise for Osc<f32> {
    type Selector = GenSel;

    fn set_param(&mut self, param: Self::Selector, val: f32) {
        match param {
            GenSel::Freq => self.set_freq(val),
            _ => (),
        }
    }
}

impl Generator for Osc<f32> {
    fn sample(&mut self) -> f32 {
        self.sample()
    }
}

impl Parametrise for WhiteNoise {
    type Selector = GenSel;

    fn set_param(&mut self, _: Self::Selector, _: f32) {}
}

impl Generator for WhiteNoise {
    fn sample(&mut self) -> f32 {
        self.sample()
    }
}

impl Parametrise for PinkNoise {
    type Selector = GenSel;

    fn set_param(&mut self, _: Self::Selector, _: f32) {}
}

impl Generator for PinkNoise {
    fn sample(&mut self) -> f32 {
        self.sample()
    }
}

impl Parametrise for BrownNoise {
    type Selector = GenSel;

    fn set_param(&mut self, _: Self::Selector, _: f32) {}
}

impl Generator for BrownNoise {
    fn sample(&mut self) -> f32 {
        self.sample()
    }
}

#[derive(Debug)]
pub enum PipelineSelector {
    Osc(usize, GenSel, f32),
    Lfo(f32),
    Mixer(usize, f32),
    Filter(usize, FilSel, f32),
}

#[derive(Debug)]
pub enum FilSel {
    Cutoff,
    Resonance,
}

#[derive(Debug)]
pub enum GenSel {
    Freq,
    Width,
}

pub trait Filter: Parametrise<Selector = FilSel> {
    fn filter(&mut self, sample: f32) -> f32;
}

impl Filter for KrajeskiLadder {
    fn filter(&mut self, sample: f32) -> f32 {
        self.filter(sample)
    }
}

impl Parametrise for KrajeskiLadder {
    type Selector = FilSel;
    fn set_param(&mut self, param: Self::Selector, val: f32) {
        match param {
            FilSel::Cutoff => self.set_cutoff(val),
            FilSel::Resonance => self.set_resonance(val),
        }
    }
}

impl Filter for MoogFilter {
    fn filter(&mut self, sample: f32) -> f32 {
        self.filter(sample)
    }
}

impl Parametrise for MoogFilter {
    type Selector = FilSel;
    fn set_param(&mut self, param: Self::Selector, val: f32) {
        match param {
            FilSel::Cutoff => self.set_cutoff(val),
            FilSel::Resonance => self.set_resonance(val),
        }
    }
}

pub struct Pipeline<L: Generator> {
    oscs: FusedGenerator,
    lfo: L,
    levels: Vec<f32>,
    adsr: ADSR<f32>,
    filters: FusedFilter,
    master: f32,
}

impl<L: Generator> Pipeline<L> {
    pub fn new(lfo: L, adsr: ADSR<f32>, master: f32) -> Self {
        Self {
            oscs: FusedGenerator::new(),
            lfo,
            levels: Vec::new(),
            adsr,
            filters: FusedFilter::new(),
            master,
        }
    }

    pub fn add_osc<G: Generator + 'static>(&mut self, osc: G, level: f32) {
        self.oscs.push(osc);
        self.levels.push(level);
    }

    pub fn add_filter<F: Filter + 'static>(&mut self, osc: F) {
        self.filters.push(osc);
    }

    pub fn trigger(&mut self) {
        self.adsr.trigger()
    }

    pub fn let_go(&mut self) {
        self.adsr.let_go()
    }

    pub fn sample(&mut self) -> f32 {
        let mut oscs = self.oscs.iter();
        let mut sample = 0.0;
        let m = self.lfo.sample();
        while let Some((i, g)) = oscs.next() {
            let modulated = g.sample() + m;
            sample += self.levels[i] * modulated;
        }
        sample = self.adsr.apply(sample);
        sample = self.filters.filter(sample);
        sample * self.master
    }

    pub fn set_param(&mut self, param: PipelineSelector) {
        match param {
            PipelineSelector::Osc(n, p, v) => self.oscs.set_param(n, p, v),
            PipelineSelector::Lfo(f) => self.lfo.set_param(GenSel::Freq, f),
            PipelineSelector::Mixer(n, l) => self.levels[n] = l,
            PipelineSelector::Filter(n, p, v) => self.filters.set_param(n, p, v),
        }
    }
}
