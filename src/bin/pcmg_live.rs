use anyhow::Result;
use cpal::{traits::*, Sample, SampleFormat};
use crossbeam_channel::{Receiver, Sender};
use pcmg::types::{Hz, Mixer, MoogFilter, Note, Oscillator, LFO};
use std::{
    marker::{Send, Sync},
    time::Instant,
};

pub enum Command {
    Stop,
}

pub enum Event {
    Started(Started),
}

pub struct Started {
    sample_rate: f32,
    _channels: usize,
    sink: Sender<f32>,
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &Receiver<f32>)
where
    T: Sample + Send + Sync,
{
    for frame in output.chunks_mut(channels) {
        let value = next_sample.recv().unwrap();
        let value = Sample::from(&value);
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

fn run<T>(
    dev: &cpal::Device,
    cfg: &cpal::StreamConfig,
    commands: Receiver<Command>,
    events: Sender<Event>,
) -> Result<()>
where
    T: Sample + Send + Sync,
{
    let sample_rate = cfg.sample_rate.0 as f32;
    let channels = cfg.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let (tx, rx) = crossbeam_channel::bounded(cfg.sample_rate.0 as usize);

    let stream = dev.build_output_stream(
        cfg,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| write_data(data, channels, &rx),
        err_fn,
    )?;
    stream.play()?;

    events.send(Event::Started(Started {
        sample_rate,
        _channels: channels,
        sink: tx,
    }))?;

    match commands.recv() {
        Ok(Command::Stop) => Ok(()),
        Err(_) => Ok(()),
    }
}

fn build_audio_thread() -> (Receiver<Event>, Sender<Command>) {
    let (command_tx, command_rx) = crossbeam_channel::unbounded();
    let (event_tx, event_rx) = crossbeam_channel::unbounded();
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let supported_config = device.default_output_config()?;
        let sample_format = supported_config.sample_format();
        let config = supported_config.into();
        match sample_format {
            SampleFormat::F32 => run::<f32>(&device, &config, command_rx, event_tx),
            SampleFormat::I16 => run::<i16>(&device, &config, command_rx, event_tx),
            SampleFormat::U16 => run::<u16>(&device, &config, command_rx, event_tx),
        }
    });
    (event_rx, command_tx)
}

fn main() -> Result<()> {
    use Note::*;
    let (ev_rx, _cmd_tx) = build_audio_thread();

    let Started {
        sample_rate,
        _channels: _,
        sink,
    } = match ev_rx.recv() {
        Ok(Event::Started(s)) => s,
        Err(e) => Err(e)?,
    };

    // let _lfo = LFO::new(90., 0.9);
    // let mixer = Mixer::new([1.0, 1.0, 1.0, 1.0]);

    // let notes = [C.f(4), D.f(4), B.f(4)];
    // let mut notes = notes.iter().cycle();

    let sine = |n: f32| n.sin();
    // let mut oscs = [
    //     Oscillator::with_freq(sample_rate, Box::new(sine), C.f(3)),
    //     Oscillator::with_freq(sample_rate, Box::new(sine), C.f(4)),
    //     Oscillator::with_freq(sample_rate, Box::new(sine), C.f(5)),
    //     Oscillator::with_freq(sample_rate, Box::new(sine), C.f(6)),
    // ];
    // let mut osc = Oscillator::new(sample_rate, Box::new(sine));
    // let mut osc = Oscillator::with_freq(sample_rate, Box::new(sine), C.f(2));
    // let mut lfo = Oscillator::with_freq(sample_rate, Box::new(sine), 440.0);

    // let mut cutoff = 0.0;
    // let mut filter = MoogFilter::new(sample_rate, cutoff, 1.0);
    // let mut filter = ResonantIIRLowpass::new(sample_rate, 200.0, 2.0);
    // let mut filter = FourPoleFilter::new_lp(440.0, 3.0);

    // let mut sample_clock = 0f32;
    let mut secs = 0.0;
    let mut next_value = move || {
        // if sample_clock < 1.0 {
        //     osc.set_freq(*notes.next().unwrap());
        // }
        // sample_clock = (sample_clock + 1.0) % (sample_rate);
        secs += 1.0 / sample_rate;
        // if cutoff < 500.0 {
        //     cutoff += 0.001;
        // }
        // filter.set_cutoff(cutoff);
        // let samples = oscs.sample(sample_clock);
        // lfo.modulate(&mut osc, secs);
        // let sample = osc.sample(secs) * 0.2;
        let sample = ((secs * 440.0).to_angular_frequency() / sample_rate).sin();
        // let sample = mixer.mix(0.2, &samples);
        // let sample = filter.filter(sample);
        sink.send(Sample::from(&sample))
    };

    loop {
        next_value()?;
    }
}
