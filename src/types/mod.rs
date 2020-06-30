#![allow(dead_code, type_alias_bounds)]
mod adsr;
mod hz;
mod lfo;
mod note;
mod write_wave;
pub use adsr::*;
pub use hz::*;
pub use lfo::*;
pub use note::*;
pub use write_wave::*;
pub type Second = f64;
pub type Sample<T: nt::Float> = T;
pub type Beat<T: nt::Float> = T;
pub type Wave<T: nt::Float> = Vec<Sample<T>>;
pub type Waveform<T: nt::Float> = fn(T, T) -> T;

//unused, possibly not needed
/*
trait ToSamples: Sized {
    fn ms_to_samples(self) -> usize;
    fn s_to_samples(self) -> usize;
}

impl ToSamples for usize {
    fn s_to_samples(self) -> usize {
        self * crate::SAMPLERATE as usize
    }
    fn ms_to_samples(self) -> usize {
        self / 1000 * crate::SAMPLERATE as usize
    }
}
*/
// TODO
pub struct Instrument {
    envelope: ADSR<f64>,
    lfo: LFO<f64>,
}
