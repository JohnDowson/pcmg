#[allow(unused_imports)]
use std::f64::consts::PI;
extern crate nom;
use itertools_num::linspace;
use std::time::Instant;
pub mod waves;
use waves::*;
pub mod trakparse;
pub mod types;
use types::*;
const PITCH: Hz = 440.; //E 329.63;
const SAMPLERATE: f64 = 48000.;
const BPM: f64 = 136.;

fn attack(i: f64, duration: usize) -> f64 {
    let _duration = duration as f64;
    let attack_length = 1000.;
    let s: f64 = (i) / (attack_length);
    let r: f64;
    if i < attack_length {
        r = lerp(0., 1., s);
    } else {
        r = 1f64;
    }
    return r;
}

fn decay(i: f64, duration: usize) -> f64 {
    let duration = duration as f64;
    let decay_length = 2000.;
    let s: f64 = ((duration - i)) / (decay_length);
    let r: f64;
    if duration - i < decay_length {
        r = lerp(-0.001, 1., s);
    } else {
        r = 1f64;
    }
    return r;
}

fn lerp(from: f64, to: f64, scalar: f64) -> f64 {
    from + (to - from) * scalar
}

fn freq(p: Note, waveform: fn(f64, f64) -> f64) -> Wave {
    match p {
        Note::Silence(duration) => return vec![0f64; (duration * SAMPLERATE) as usize],
        Note::Note(hz, duration) => {
            let total_samples = (duration * SAMPLERATE) as usize;
            let volume = 0.5;
            return linspace::<f64>(0., duration * SAMPLERATE, total_samples)
                .enumerate()
                .map(|x| {
                    let (_step, t) = x;
                    let mut sample = waveform(hz, t);
                    sample = (attack(t, total_samples) * sample) * decay(t, total_samples);
                    sample * volume
                })
                .map(|x| x as f64)
                .collect();
        }
    }
}

fn write(p: &str, w: Wave) {
    use byteorder::{ByteOrder, LittleEndian};
    use std::fs::File;
    use std::io::Write;
    use std::mem::size_of;
    println!("Writing to file {}", p);
    let mut f = File::create(p).expect("Can't create specified file");
    let mut b = vec![0u8; size_of::<Sample>() * w.len()];
    LittleEndian::write_f64_into(&w, &mut b);
    f.write_all(&b).expect("Can't write to specified file");
}

fn main() {
    let t1 = Instant::now();
    let wave: &[Interval];
    /*wave = &[
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.5),
        Interval::Note(12., 0.5),
        Interval::Note(10., 0.25),
        Interval::Note(10., 0.5),
        Interval::Note(10., 0.5),
        Interval::Note(5., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(12., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(12., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.5),
        Interval::Note(10., 0.25),
        Interval::Note(10., 0.25),
        Interval::Note(10., 0.25),
        Interval::Note(10., 0.25),
        Interval::Note(10., 0.25),
        Interval::Note(10., 0.25),
        Interval::Note(10., 0.5),
        Interval::Note(5., 0.25),
        Interval::Note(5., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.25),
        Interval::Hold(1.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(10., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(10., 0.5),
        Interval::Note(10., 0.5),
        Interval::Hold(2.),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.25),
        Interval::Note(12., 0.5),
        Interval::Note(12., 0.5),
        Interval::Note(10., 0.25),
        Interval::Note(10., 0.25),
        Interval::Note(10., 0.25),
        Interval::Note(10., 0.25),
        Interval::Note(10., 0.5),
        Interval::Note(10., 0.5),
        Interval::Note(5., 0.25),
        Interval::Note(5., 0.25),
        Interval::Hold(1.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(10., 0.5),
        Interval::Note(10., 0.5),
        Interval::Hold(4.),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(10., 0.5),
        Interval::Note(10., 0.5),
        Interval::Hold(4.),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(10., 0.5),
        Interval::Note(10., 0.5),
        Interval::Hold(4.),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.5),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.25),
        Interval::Note(7., 0.5),
        Interval::Note(10., 0.5),
        Interval::Note(10., 0.5),
        Interval::Hold(4.),
    ];*/
    wave = &[Interval::Note(0., 10.)];
    let wave = wave.iter().map(|x| x.to_note(BPM, PITCH));
    write(
        &"out.bin",
        wave.map(|i| {
            freq(i, |f, t| {
                //
                //sine(f*2., t) + sine(f/2., t) + sine(f, t)
                lerp(sine(f*2., t), sine(f/2., t), sine(f, fa(t)))
            })
        })
        .flatten()
        .collect(),
    );
    println!("{:?}", t1.elapsed());
}
