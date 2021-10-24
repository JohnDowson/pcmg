use super::Hz;
#[derive(Debug, Clone, Copy)]
pub struct Note<F, D>
where
    F: num::Float + Hz<F>,
    D: num::Float,
{
    pub silent: bool,
    pub freq: F,
    pub duration: D,
}
impl<F, D> Note<F, D>
where
    F: num::Float + Hz<F>,
    D: num::Float,
{
    pub fn new(freq: F, duration: D) -> Note<F, D> {
        Note {
            silent: false,
            freq: freq,
            duration: duration,
        }
    }
    pub fn silent(duration: D) -> Note<F, D> {
        Note {
            silent: true,
            freq: F::from(0.0).unwrap(),
            duration: duration,
        }
    }
    // FIXME
    pub fn beat_to_second(b: f64, bpm: f64) -> f64 {
        b * (60. / bpm)
    }
}
