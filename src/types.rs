pub type Semitone = f64;
pub type Hz = f64;
pub type Second = f64;
pub type Sample = f64;
pub type Beat = f64;
pub type Wave = Vec<Sample>;
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Interval {
    Note(Semitone, Beat),
    Hold(Beat),
}
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Note {
    Note(Hz, Second),
    Silence(Second),
}
#[allow(dead_code)]
impl Interval {
    pub fn to_note(self, bpm: f64, base_pitch: f64) -> Note {
        Note::from_interval(self, bpm, base_pitch)
    }
}
#[allow(dead_code)]
impl Note {
    fn from_interval(interval: Interval, bpm: f64, base_pitch: f64) -> Note {
        match interval {
            Interval::Hold(beat) => Note::Silence(beat_to_second(beat, bpm)),
            Interval::Note(s, beat) => {
                Note::Note(semitone_to_hz(s, base_pitch), beat_to_second(beat, bpm))
            }
        }
    }
}
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
fn semitone_to_hz(s: Semitone, base_pitch: Hz) -> Hz {
    let a: f64 = (2.0 as f64).powf(1. / 12.);
    return base_pitch * a.powf(s);
}

fn beat_to_second(b: Beat, bpm: f64) -> Second {
    b * (60. / bpm)
}
