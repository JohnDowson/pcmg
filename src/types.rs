pub type Semitone = f64;
pub type Hz = f64;
pub type Second = f64;
pub type Sample<T: num::Float> = T;
pub type Beat = f64;
pub type Wave<T: num::Float> = Vec<Sample<T>>;
//pub type Wavedef = fn(Hz) -> Waveform;
pub type Waveform<T: num::Float> = fn(T, T, &mut Arpeggiator<T>) -> T;
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
enum Direction {
    Forward,
    Back,
}
impl Direction {
    pub fn switch(&self) -> Direction {
        match self {
            Direction::Forward => Direction::Back,
            Direction::Back => Direction::Forward,
        }
    }
}
#[allow(dead_code)]
pub struct Arpeggiator<T: Sized> {
    seq: Vec<T>,
    current: usize,
    next: usize,
    direction: Direction,
    step_by: T,
    step: T,
    initialized: bool,
}
#[allow(dead_code)]
impl Arpeggiator<f64> {
    pub fn new_uninitialized() -> Arpeggiator<f64> {
        Arpeggiator::<f64> {
            seq: vec![],
            current: 0,
            next: 0,
            direction: Direction::Forward,
            step_by: 0.,
            step: 0.,
            initialized: false,
        }
    }
    pub fn init(&mut self, seq: Vec<f64>, step_by: f64) -> Result<(), &'static str> {
        match seq.len() {
            0..=1 => Err("Sequence must have two or more values"),
            _ => {
                self.seq = seq.clone();
                self.current = 0;
                self.next = 1;
                self.direction = Direction::Forward;
                self.step_by = step_by;
                self.step = 0.;
                self.initialized = true;
                Ok(())
            }
        }
    }
    pub fn new(seq: Vec<f64>, step_by: f64) -> Result<Arpeggiator<f64>, &'static str> {
        match seq.len() {
            0..=1 => Err("Sequence must have two or more values"),
            _ => Ok(Arpeggiator::<f64> {
                seq: seq.clone(),
                current: 0,
                next: 1,
                direction: Direction::Forward,
                step_by: step_by,
                step: 0.,
                initialized: true,
            }),
        }
    }
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    // neither does this
    /* pub fn next(&mut self) -> f64 {
        let r: f64;
        if self.step >= 1. {
            r = self.seq[self.next];
            if self.next + 1 == self.seq.len() {
                self.current = 0;
                self.next = 1;
                self.step = 0.;
            } else {
                self.current += 1;
                self.next += 1;
                self.step = 0.;
            }
        } else {
            r = self.seq[self.current];
            //go on
            self.step += self.step_by
        }
        return r
    } */
    // this monstrocity does something that might end up useful, but totally not what i intended
    /* pub fn next(&mut self) -> f64 {
        let r = crate::lerp(self.seq[self.current], self.seq[self.next], self.step);
        if self.step >= 1. {
            //next target or change direction
            match self.direction {
                Direction::Forward => {
                    if self.next + 1 == self.seq.len() {
                        self.direction = self.direction.switch();
                        // switch current and target around
                        self.current += 1;
                        self.next -= 1;
                        self.step = 0.;
                    } else {
                        self.current += 1;
                        self.next += 1;
                        self.step = 0.;
                    }
                }
                Direction::Back => {
                    if self.next == 0 {
                        self.direction = self.direction.switch();
                        self.current -= 1;
                        self.next += 1;
                        self.step = 0.;
                    } else {
                        self.current -= 1;
                        self.next -= 1;
                        self.step = 0.;
                    }
                }
            }
        } else {
            //go on
            self.step += self.step + self.step_by
        }
        return r;
    } */
}

pub trait WriteWave<T: num::Float> {
    fn write(&self, p: &str);
}
impl WriteWave<f64> for Wave<f64> {
    fn write(&self, p: &str) {
        use byteorder::{ByteOrder, LittleEndian};
        use std::fs::File;
        use std::io::Write;
        use std::mem::size_of;
        println!("Writing to file {}", p);
        let mut f = File::create(p).expect("Can't create specified file");
        let mut b = vec![0u8; size_of::<Sample<f64>>() * self.len()];
        LittleEndian::write_f64_into(self, &mut b);
        f.write_all(&b).expect("Can't write to specified file");
    }
}
impl WriteWave<f32> for Wave<f32> {
    fn write(&self, p: &str) {
        use byteorder::{ByteOrder, LittleEndian};
        use std::fs::File;
        use std::io::Write;
        use std::mem::size_of;
        println!("Writing to file {}", p);
        let mut f = File::create(p).expect("Can't create specified file");
        let mut b = vec![0u8; size_of::<Sample<f32>>() * self.len()];
        LittleEndian::write_f32_into(&self, &mut b);
        f.write_all(&b).expect("Can't write to specified file");
    }
}

fn semitone_to_hz(s: Semitone, base_pitch: Hz) -> Hz {
    let a: f64 = (2.0 as f64).powf(1. / 12.);
    return base_pitch * a.powf(s);
}

fn beat_to_second(b: Beat, bpm: f64) -> Second {
    b * (60. / bpm)
}
