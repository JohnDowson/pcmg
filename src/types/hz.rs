use crate::consts::*;
pub trait Hz<F: num::Float> {
    fn to_angular_frequency(self) -> Self;
    fn from_midi_note_id(note_id: Self) -> Self;
    fn from_semitone(semitone: Self) -> Self;
    fn from_semitone_c(semitone: Self) -> Self;
}

impl Hz<f64> for f64 {
    fn to_angular_frequency(self) -> f64 {
        self * F64::TWO_PI
    }
    fn from_midi_note_id(note_id: f64) -> f64 {
        Self::from_semitone(note_id - 69.0)
    }
    fn from_semitone(semitone: f64) -> f64 {
        F64::A4 * F64::TWELFTH_ROOT_OF_2.powf(semitone)
    }
    fn from_semitone_c(semitone: f64) -> f64 {
        2.0f64.powf(semitone / 12.) * F64::A4
    }
}

impl Hz<f32> for f32 {
    fn to_angular_frequency(self) -> f32 {
        self * F32::TWO_PI
    }
    fn from_midi_note_id(note_id: f32) -> f32 {
        Self::from_semitone(note_id - 69.0)
    }
    fn from_semitone(semitone: f32) -> f32 {
        F32::A4 * F32::TWELFTH_ROOT_OF_2.powf(semitone)
    }
    fn from_semitone_c(semitone: f32) -> f32 {
        2.0f32.powf(semitone / 12.) * F32::A4
    }
}
