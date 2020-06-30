#![allow(unused_imports, dead_code)]
extern crate num_traits as nt;
use std::f64::consts::PI;
use std::time::Instant;
pub mod types;
pub mod waves;
use types::*;
pub mod consts;
mod live_output;
use consts::*;

fn freq(p: &Note<f64, f64>, waveform: fn(f64) -> f64) -> Wave<f64> {
    match p {
        Note {
            silent: true,
            duration,
            ..
        } => return vec![0f64; (duration * SAMPLERATE) as usize],
        Note {
            silent: false,
            freq,
            duration,
        } => {
            let volume = 0.1;
            let lfo = LFO::new(0., 0.);
            return (0..=(duration * SAMPLERATE) as usize)
                .map(|x| x as f64 / SAMPLERATE)
                .map(|t| {
                    let attack = 0.01;
                    let decay = 0.1;
                    let mut alpha = if t <= attack {
                        //attack
                        t / attack
                    } else if t >= (duration - decay) {
                        //release
                        (duration - t) / decay
                    } else {
                        //sustain
                        1.
                    };
                    alpha *= 1.0;
                    (waveform(lfo.apply(*freq, t)) * alpha) * volume
                })
                .collect();
        }
    }
}

fn main() {
    let outp = std::env::args()
        .find(|a| a.ends_with(".bin"))
        .expect("No valid output file specified");
    let t1 = Instant::now();
    /* let wave = &[
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.5),
        Note::new(12., 0.5),
        Note::new(10., 0.25),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::new(5., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.5),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.5),
        Note::new(5., 0.25),
        Note::new(5., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::silent(1.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(2.),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.5),
        Note::new(12., 0.5),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::new(5., 0.25),
        Note::new(5., 0.25),
        Note::silent(1.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(4.),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(4.),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(4.),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(4.),
    ]; */
    let wave = &[Note::new(440., 1.), Note::new(450., 1.)];
    wave.iter()
        .map(|note| {
            freq(note, |hertz| {
                use std::f64;
                f64::sin(hertz)
            })
        })
        .flatten()
        .collect::<Wave<f64>>()
        .write(&outp);
    println!("{:?}", t1.elapsed());
}

/* fn mkfn(i: usize) -> impl Fn((f64, f64)) -> (f64, f64) {
    move |input| (input.0 * i as f64, input.1)
}
fn foo() -> f64 {
    let hz: f64 = 440.;
    let t: f64 = 0.1;
    let chains = vec![
        [mkfn(1), mkfn(2), mkfn(3), mkfn(4), mkfn(5)],
        [mkfn(1), mkfn(2), mkfn(3), mkfn(4), mkfn(5)],
        [mkfn(1), mkfn(2), mkfn(3), mkfn(4), mkfn(5)],
    ];
    chains
        .iter()
        .map(|c| c.iter().fold((hz, t), |input: (f64, f64), fun| fun(input)))
        .map(|c: (f64, f64)| c.0)
        .sum()
}

extern crate anyhow;
extern crate cpal;

use cpal::traits::*;

pub fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    let event_loop = host.event_loop();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let mut supported_formats_range = device
        .supported_output_formats()
        .expect("error while querying formats");
    let format = supported_formats_range
        .next()
        .expect("no supported format?!")
        .with_max_sample_rate();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop
        .play_stream(stream_id)
        .expect("failed to play_stream");
    use cpal::{StreamData, UnknownTypeOutputBuffer};
    let sample_rate = format.sample_rate.0 as f32;
    fn w(hz: f32) -> f32 {
        const TWO_PI: f32 = 2. * std::f32::consts::PI;
        hz * TWO_PI
    }
    fn apply_lfo(hz: f32, time: f32) -> f32 {
        const AMP: f32 = 0.01;
        const FREQ: f32 = 5.;
        w(hz) * time + AMP * hz * (w(FREQ) * time).sin()
    }
    let step = 1.0 / sample_rate;
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + step) % 1.;
        //(apply_lfo(440., (sample_clock + 0.) / sample_rate)).sin()
        (sample_clock * w(440.) / sample_rate).sin()
    };
    event_loop.run(move |stream_id, stream_result| {
        let stream_data = match stream_result {
            Ok(data) => data,
            Err(err) => {
                eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                return;
            }
        };
        match stream_data {
            StreamData::Output {
                buffer: UnknownTypeOutputBuffer::U16(mut buffer),
            } => {
                for elem in buffer.iter_mut() {
                    *elem = u16::max_value() / 2;
                }
            }
            StreamData::Output {
                buffer: UnknownTypeOutputBuffer::I16(mut buffer),
            } => {
                for elem in buffer.iter_mut() {
                    *elem = 0;
                }
            }
            StreamData::Output {
                buffer: UnknownTypeOutputBuffer::F32(mut buffer),
            } => {
                println!("{:?}", buffer.len());
                for elem in buffer.iter_mut() {
                    *elem = cpal::Sample::from::<f32>(&(0.1 * next_value()));
                }
            }
            _ => (),
        }
    });
}
 */
