use super::Hz;
pub struct LFO<T: Hz<T> + nt::Float> {
    freq: T,
    amplitue: T,
}
impl<T: Hz<T> + nt::Float> LFO<T> {
    pub fn new(freq: T, amplitue: T) -> LFO<T> {
        LFO {
            freq: freq,
            amplitue: amplitue,
        }
    }
    pub fn none_lfo() -> LFO<T> {
        LFO {
            freq: T::from(0.).unwrap(),
            amplitue: T::from(0.).unwrap(),
        }
    }
    pub fn apply(&self, hz: T, time: T) -> T {
        hz.w() * time + self.amplitue * hz * (self.freq.w() * time).sin()
    }
}
