extern crate nom;
use byteorder::{ByteOrder, LittleEndian};
use itertools_num::linspace;
use std::f64::consts::PI;
use std::fs::File;
use std::io::prelude::*;
use std::mem::size_of;
use std::time::Instant;
pub mod waves;
use waves::*;
pub mod trakparse;
pub mod types;
use types::*;
const PITCH: Hz = 440.; //E 329.63;
const SAMPLERATE: f64 = 48000.;
const BPM: f64 = 136.;

fn attack(i: usize, _duration: usize) -> f64 {
    let attack_length = 1000;
    let s: f64 = (i as f64) / (attack_length as f64);
    let r: f64;
    if i < attack_length {
        r = lerp(0., 1., s);
    } else {
        r = 1f64;
    }
    return r;
}

fn decay(i: usize, duration: usize) -> f64 {
    let decay_length = 1000;
    let s: f64 = ((duration - i) as f64) / (decay_length as f64);
    let r: f64;
    if duration - i < decay_length {
        r = lerp(0., 1., s);
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
            //let freq = (2. * PI * hz) / SAMPLERATE; //((hz * 2.0) as f64 * PI) / SAMPLERATE;
            return linspace::<f64>(0., duration * SAMPLERATE, total_samples)
                .enumerate()
                .map(|x| {
                    let (step, t) = x;
                    let mut sample = waveform(hz, t);
                    sample = (attack(step, total_samples) * sample) * decay(step, total_samples);
                    sample * volume
                })
                .map(|x| x as f64)
                .collect();
        }
    }
}

fn write(p: &str, w: Wave) {
    println!("Writing to file");
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
    //use rand::prelude::*;
    write(
        &"out.bin",
        wave.map(|i| {
            freq(
                i,
                //|x| (phat_sine(x / 1.5) + squeaky(x * 1.5) / 2. + square(x / 2.) + noize(x)) / 4.
                //|x| (phat_sine(x/2.)/2. + squeaky(x/1.5)/2. + square(x)/2.)
                //|x| *[sine(-x), sawtooth(x)].choose(&mut thread_rng()).unwrap()
                |f, t| sine(sine(-f, t), t),
            )
        })
        .flatten()
        .collect(),
    );
    println!("{:?}", t1.elapsed());
}
