#![allow(dead_code)]
mod adsr;
mod errors;
pub mod filters;
pub mod generators;
mod hz;
mod note;

use self::{
    filters::{KrajeskiLadder, MoogFilter},
    generators::{BrownNoise, FmOsc, Osc, PinkNoise, SquarePulse, WhiteNoise},
};
pub use adsr::*;
pub use errors::*;
use fusebox::FuseBox;
pub use hz::*;
pub use note::*;
use std::ops::RangeInclusive;

pub struct FusedGenerator {
    inner: FuseBox<dyn Generator>,
}

impl FusedGenerator {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn set_param(&mut self, n: usize, param: GenSel, val: f32) {
        self.get(n).set_param(param, val)
    }

    pub fn push<G: Generator + 'static>(&mut self, g: G) {
        fusebox::push!(g, self.inner, Generator);
    }

    pub fn get(&mut self, n: usize) -> &mut dyn Generator {
        self.inner.get_mut(n)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct FusedFilter {
    inner: FuseBox<dyn Filter>,
}

impl FusedFilter {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn filter(&mut self, sample: f32) -> f32 {
        self.inner
            .iter_mut()
            .fold(sample, |sample, filter| filter.filter(sample))
    }

    pub fn set_param(&mut self, n: usize, param: FilSel, val: f32) {
        self.get(n).set_param(param, val)
    }

    pub fn push<G: Filter + 'static>(&mut self, g: G) {
        fusebox::push!(g, self.inner, Filter);
    }

    pub fn get(&mut self, n: usize) -> &mut dyn Filter {
        self.inner.get_mut(n)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

pub trait Generator: Parametrise<Selector = GenSel> {
    fn sample(&mut self) -> f32;
}

pub trait Parametrise {
    type Selector;
    fn set_param(&mut self, _param: Self::Selector, _val: f32) {}
    fn list_params(&self) -> Vec<(Self::Selector, RangeInclusive<f32>)> {
        vec![]
    }
}

impl Parametrise for SquarePulse<f32> {
    type Selector = GenSel;

    fn set_param(&mut self, param: Self::Selector, val: f32) {
        match param {
            GenSel::Freq => self.set_freq(val),
            GenSel::Detune => self.set_detune(val),
            GenSel::Width => self.set_width(val),
            _ => (),
        }
    }

    fn list_params(&self) -> Vec<(Self::Selector, RangeInclusive<f32>)> {
        vec![
            (GenSel::Freq, 0.0..=1000.0),
            (GenSel::Detune, -500.0..=500.0),
            (GenSel::Width, 0.0..=1.0),
        ]
    }
}

impl Generator for SquarePulse<f32> {
    fn sample(&mut self) -> f32 {
        self.sample()
    }
}

impl Parametrise for FmOsc<f32> {
    type Selector = GenSel;

    fn set_param(&mut self, param: Self::Selector, val: f32) {
        match param {
            GenSel::Freq => self.set_freq(val),
            GenSel::FmRatio(n) => self.set_fm_ratio(n, val),
            GenSel::FmIndex(n) => self.set_fm_index(n, val),
            _ => (),
        }
    }

    fn list_params(&self) -> Vec<(Self::Selector, RangeInclusive<f32>)> {
        vec![
            (GenSel::Freq, 0.0..=1000.0),
            (GenSel::FmRatio(0), 0.0..=1000.0),
            (GenSel::FmIndex(0), 0.0..=1000.0),
            (GenSel::FmRatio(1), 0.0..=1000.0),
            (GenSel::FmIndex(1), 0.0..=1000.0),
        ]
    }
}

impl Generator for FmOsc<f32> {
    fn sample(&mut self) -> f32 {
        self.sample()
    }
}

impl Parametrise for Osc<f32> {
    type Selector = GenSel;

    fn set_param(&mut self, param: Self::Selector, val: f32) {
        match param {
            GenSel::Freq => self.set_freq(val),
            GenSel::Detune => self.set_detune(val),
            _ => (),
        }
    }

    fn list_params(&self) -> Vec<(Self::Selector, RangeInclusive<f32>)> {
        vec![
            (GenSel::Freq, 0.0..=1000.0),
            (GenSel::Detune, -500.0..=500.0),
        ]
    }
}

impl Generator for Osc<f32> {
    fn sample(&mut self) -> f32 {
        self.sample()
    }
}

impl Parametrise for WhiteNoise {
    type Selector = GenSel;
}

impl Generator for WhiteNoise {
    fn sample(&mut self) -> f32 {
        self.sample()
    }
}

impl Parametrise for PinkNoise {
    type Selector = GenSel;
}

impl Generator for PinkNoise {
    fn sample(&mut self) -> f32 {
        self.sample()
    }
}

impl Parametrise for BrownNoise {
    type Selector = GenSel;
}

impl Generator for BrownNoise {
    fn sample(&mut self) -> f32 {
        self.sample()
    }
}

#[derive(Debug)]
pub enum PipelineSelector {
    Osc(Option<usize>, GenSel, f32),
    Lfo(LfoSel, f32),
    Distortion(bool),
    Mixer(usize, f32),
    Master(f32),
    Filter(usize, FilSel, f32),
}

#[derive(Debug, Clone, Copy)]
pub enum LfoSel {
    Freq,
    Depth,
}

#[derive(Debug, Clone, Copy)]
pub enum FilSel {
    Cutoff,
    Resonance,
}

#[derive(Debug, Clone, Copy)]
pub enum GenSel {
    Freq,
    FmRatio(usize),
    FmIndex(usize),
    Detune,
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

    fn list_params(&self) -> Vec<(Self::Selector, RangeInclusive<f32>)> {
        vec![
            (FilSel::Cutoff, 0.0..=1000.0),
            (FilSel::Resonance, 0.0..=2.0),
        ]
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

    fn list_params(&self) -> Vec<(Self::Selector, RangeInclusive<f32>)> {
        vec![
            (FilSel::Cutoff, 0.0..=1000.0),
            (FilSel::Resonance, 0.0..=2.0),
        ]
    }
}

pub struct Pipeline<L: Generator> {
    oscs: FusedGenerator,
    lfo: L,
    mod_depth: f32,
    distortion: bool,
    levels: Vec<f32>,
    adsr: ADSR<f32>,
    filters: FusedFilter,
    master: f32,
}

impl<L: Generator> Pipeline<L> {
    pub fn new(lfo: L, adsr: ADSR<f32>) -> Self {
        Self {
            oscs: FusedGenerator::new(),
            lfo,
            mod_depth: 0.0,
            distortion: false,
            levels: Vec::new(),
            adsr,
            filters: FusedFilter::new(),
            master: 1.0,
        }
    }

    pub fn add_osc<G: Generator + 'static>(
        &mut self,
        osc: G,
        level: f32,
    ) -> (usize, Vec<(GenSel, RangeInclusive<f32>)>) {
        let params = osc.list_params();
        let n = self.oscs.len();
        self.oscs.push(osc);
        self.levels.push(level);
        (n, params)
    }

    pub fn add_filter<F: Filter + 'static>(
        &mut self,
        filter: F,
    ) -> (usize, Vec<(FilSel, RangeInclusive<f32>)>) {
        let params = filter.list_params();
        let n = self.filters.len();
        self.filters.push(filter);
        (n, params)
    }

    pub fn trigger(&mut self) {
        self.adsr.trigger()
    }

    pub fn hold(&mut self) {
        self.adsr.hold()
    }

    pub fn let_go(&mut self) {
        self.adsr.let_go()
    }

    pub fn sample(&mut self) -> f32 {
        let mut oscs = self.oscs.inner.iter_mut().enumerate();
        let mut sample = 0.0;
        let m = self.lfo.sample() * self.mod_depth;
        while let Some((i, g)) = oscs.next() {
            let modulated = g.sample() + m;
            sample += modulated * self.levels[i];
        }
        if self.distortion {
            sample = sample.powi(3).tanh();
        }
        sample = self.filters.filter(sample);
        sample = self.adsr.apply(sample);
        sample * self.master
    }

    pub fn set_param(&mut self, param: PipelineSelector) {
        match param {
            PipelineSelector::Osc(Some(n), p, v) => self.oscs.set_param(n, p, v),
            PipelineSelector::Osc(None, p, v) => {
                for n in 0..self.oscs.len() {
                    self.oscs.set_param(n, p, v)
                }
            }
            PipelineSelector::Distortion(d) => self.distortion = d,
            PipelineSelector::Lfo(LfoSel::Freq, f) => self.lfo.set_param(GenSel::Freq, f),
            PipelineSelector::Lfo(LfoSel::Depth, d) => self.mod_depth = d,
            PipelineSelector::Mixer(n, l) => self.levels[n] = l,
            PipelineSelector::Master(l) => self.master = l,
            PipelineSelector::Filter(n, p, v) => self.filters.set_param(n, p, v),
        }
    }
}
