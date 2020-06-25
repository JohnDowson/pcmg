use byteorder::{ByteOrder, LittleEndian};
use itertools_num::linspace;
use std::f64::consts::PI;
use std::fs::File;
use std::io::prelude::*;
use std::mem::size_of;
use std::time::Instant;
mod waves;
use waves::*;
type Hz = f64;
type Second = f64;
type Sample = f64;
type Beat = f64;
type Wave = Vec<Sample>;

fn freq(p: (Option<Hz>, Second), waveform: fn(f64) -> f64) -> Wave {
    match p {
        (None, duration) => return vec![0f64; (duration * SAMPLERATE) as usize],
        (Some(hz), duration) => {
            let total_samples = (duration * SAMPLERATE) as usize;
            println!("{:?}", p);
            let volume = 0.5;
            let step = ((hz * 2.0) as f64 * PI) / SAMPLERATE;
            return linspace::<f64>(0., duration * SAMPLERATE, total_samples)
                .enumerate()
                .map(|x| {
                    let mut step = waveform(x.1 * step);
                    step = (attack(x.0, total_samples) * step) * decay(x.0, total_samples);
                    step * volume
                })
                .map(|x| x as f64)
                .collect();
        }
    }
}

fn beat(p: (Hz, Beat)) -> (Hz, Second) {
    let (hz, beats) = p;
    (hz, (beats * (60. / BPM)))
}

fn somefy(p: (Hz, Beat)) -> (Option<Hz>, Second) {
    let (hz, beats) = p;
    (Some(hz), beats)
}

fn write(p: &str, w: Wave) {
    println!("Writing to file");
    let mut f = File::create(p).expect("Can't create specified file");
    let mut b = vec![0u8; size_of::<Sample>() * w.len()];
    LittleEndian::write_f64_into(&w, &mut b);
    f.write_all(&b).expect("Can't write to specified file");
}
const PITCH: Hz = 329.63;
const SAMPLERATE: f64 = 48000.;
const BPM: f64 = 136.;
fn f(n: f64) -> Hz {
    let a: f64 = (2.0 as f64).powf(1. / 12.);
    return PITCH * a.powf(n);
}

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

trait ToSamples: Sized {
    fn ms_to_samples(self) -> usize;
    fn s_to_samples(self) -> usize;
}

impl ToSamples for usize {
    fn s_to_samples(self) -> usize {
        self * SAMPLERATE as usize
    }
    fn ms_to_samples(self) -> usize {
        self / 1000 * SAMPLERATE as usize
    }
}

fn main() {
    let t1 = Instant::now();
    let wave: &[(Hz, Beat)];
    wave = &[
        (7., 0.5),
        (7., 0.5),
        (7., 0.25),
        (7., 0.5),
        (7., 0.5),
        (12., 0.25),
        (12., 0.5),
        (12., 0.5),
        (10., 0.25),
        (10., 0.5),
        (10., 0.5),
        (5., 0.25),
        (7., 0.5),
        (7., 0.5),
        (7., 0.25),
        (7., 0.5),
        (7., 0.5),
        (12., 0.25),
        (7., 0.5),
        (7., 0.5),
        (7., 0.25),
        (7., 0.5),
        (7., 0.5),
        (12., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (12., 0.25),
        (12., 0.25),
        (12., 0.25),
        (12., 0.25),
        (12., 0.25),
        (12., 0.25),
        (12., 0.5),
        (10., 0.25),
        (10., 0.25),
        (10., 0.25),
        (10., 0.25),
        (10., 0.25),
        (10., 0.25),
        (10., 0.5),
        (5., 0.25),
        (5., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (12., 0.25),
        (12., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (12., 0.25),
        (12., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (7., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (7., 0.5),
        (10., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (7., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (10., 0.5),
        (10., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (7., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (7., 0.5),
        (12., 0.25),
        (12., 0.25),
        (12., 0.25),
        (12., 0.25),
        (12., 0.5),
        (12., 0.5),
        (10., 0.25),
        (10., 0.25),
        (10., 0.25),
        (10., 0.25),
        (10., 0.5),
        (10., 0.5),
        (5., 0.25),
        (5., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (7., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (10., 0.5),
        (10., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (7., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (10., 0.5),
        (10., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (7., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (10., 0.5),
        (10., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (7., 0.5),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.25),
        (7., 0.5),
        (10., 0.5),
        (10., 0.5),
    ];
    let wave = wave.iter().map(|x| (f(x.0), x.1));
    write(
        &"out.bin",
        wave.map(beat)
            .map(somefy)
            .map(|i| {
                freq(i, {
                    |x| (phat_sine(x / 1.5) + squeaky(x * 1.5) / 2. + square(x / 2.)) / 4.
                })
            })
            .flatten()
            .collect(),
    );
    println!("{:?}", t1.elapsed());
}
