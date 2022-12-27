use num_traits::{Float, FromPrimitive, Zero};

use super::Hz;

#[derive(Debug, Clone, Copy)]
pub enum Note {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

impl Note {
    pub fn o(self, o: u8, d: f32) -> MidiNote {
        MidiNote::from_note_octave(self, o, d)
    }

    pub fn f<T: Float + Hz<T> + FromPrimitive>(self, o: u8) -> T {
        MidiNote::from_note_octave(self, o, 0.0).to_hz()
    }
}

#[derive(Clone, Copy)]
pub struct MidiNote {
    number: Option<u8>,
    duration: f32,
}

impl std::fmt::Debug for MidiNote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(n) = self.number {
            use Note::*;
            const NOTES: [Note; 12] = [A, As, B, C, Cs, D, Ds, E, F, Fs, G, Gs];

            let n = n - 21;
            let octave = (f32::from(n) / 12.0).ceil() as usize;
            let note = NOTES[(n % 12) as usize];
            write!(f, "{:?}{} {}", note, octave, self.duration)
        } else {
            write!(f, "Silent {}", self.duration)
        }
    }
}

impl MidiNote {
    pub fn from_midi(number: u8, duration: f32) -> Self {
        Self {
            number: Some(number),
            duration,
        }
    }

    pub fn silent(duration: f32) -> Self {
        Self {
            number: None,
            duration,
        }
    }

    pub fn from_note_octave(n: Note, o: u8, d: f32) -> Self {
        let nr = match n {
            Note::A => 0,
            Note::As => 1,
            Note::B => 2,
            Note::C => 3,
            Note::Cs => 4,
            Note::D => 5,
            Note::Ds => 6,
            Note::E => 7,
            Note::F => 8,
            Note::Fs => 9,
            Note::G => 10,
            Note::Gs => 11,
        };
        let nr = 21 + nr + (f32::from(o) * 12.0).floor() as u8;

        Self::from_midi(nr, d)
    }

    pub fn to_hz<T: Float + FromPrimitive + Zero + Hz<T>>(&self) -> T {
        if let Some(n) = self.number {
            T::from_midi_note_id(T::from_u8(n).unwrap())
        } else {
            T::zero()
        }
    }
}

#[test]
fn test() {
    dbg! {MidiNote::from_midi(69, 0.0)};

    dbg! {MidiNote::from_note_octave(Note::A, 4, 0.0)};
}
