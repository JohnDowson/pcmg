use super::Hz;
pub struct LFO<T: Hz<T> + num_traits::Float> {
    freq: T,
    amplitue: T,
}
impl<T: Hz<T> + num_traits::Float> LFO<T> {
    pub fn new(freq: T, amplitue: T) -> LFO<T> {
        LFO { freq, amplitue }
    }
    pub fn none_lfo() -> LFO<T> {
        LFO {
            freq: T::from(0.).unwrap(),
            amplitue: T::from(0.).unwrap(),
        }
    }
    pub fn apply(&self, hz: T, time: T) -> T {
        hz.to_angular_frequency() * time
            + self.amplitue * hz * (self.freq.to_angular_frequency() * time).sin()
    }
}
