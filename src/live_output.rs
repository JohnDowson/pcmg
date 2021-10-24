#![allow(dead_code)]
extern crate cpal;
use cpal::traits::*;
use crossbeam_channel::{Receiver, Sender};
use std::marker::{Send, Sync};
pub struct Event {}

pub fn main<T: cpal::Sample + Send + Sync>(
    rx_sample: Receiver<&[T; 960]>,
    rx_events: Receiver<Event>,
    tx_format: Sender<cpal::Format>,
) -> Result<(), anyhow::Error> {
    // initialize
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
    // send format to the synthesizer thread
    // or actually dont, we can do synthesis in f32 and then just convert samples into correct format using cpal::Sample::from
    // nvm we still need the sample rate
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
    // start event loop
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
                // try getting batch of samples from synthesizer
            } => match rx_sample.try_iter().next() {
                /*
                From testing it seems that the buffer is 960 samples long,
                but I have no idea if i can depend on it always being that size
                Neither do I know if there is a way to control buffer size
                */
                Some(x) => {
                    for elem in buffer.iter_mut() {
                        // write samples into provided buffer
                        // FIXME: this is exceptionally ugly
                        *elem = x.iter().next().unwrap_or(&(T::from(&0.0))).to_u16()
                    }
                }
                None => {
                    for elem in buffer.iter_mut() {
                        *elem = num::identities::zero()
                    }
                }
            },
            StreamData::Output {
                buffer: UnknownTypeOutputBuffer::I16(mut buffer),
            } => match rx_sample.try_iter().next() {
                Some(x) => {
                    for elem in buffer.iter_mut() {
                        *elem = x.iter().next().unwrap_or(&(T::from(&0.0))).to_i16()
                    }
                }
                None => {
                    for elem in buffer.iter_mut() {
                        *elem = num::identities::zero()
                    }
                }
            },
            StreamData::Output {
                buffer: UnknownTypeOutputBuffer::F32(mut buffer),
            } => match rx_sample.try_iter().next() {
                Some(x) => {
                    for elem in buffer.iter_mut() {
                        *elem = x.iter().next().unwrap_or(&(T::from(&0.0))).to_f32()
                    }
                }
                None => {
                    for elem in buffer.iter_mut() {
                        *elem = num::identities::zero()
                    }
                }
            },
            _ => (),
        }
    });
}
