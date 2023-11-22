#![feature(ptr_metadata)]

pub mod compiled_graph;
pub mod consts;
pub mod devices;
pub mod graph;
pub mod types;
pub mod waves;
pub mod widgets;

use anyhow::{anyhow, Result};
use cpal::{traits::*, Sample, SampleFormat};
use crossbeam_channel::{Receiver, Sender};
use midir::MidiInput;
use wmidi::{FromBytesError, MidiMessage};
pub enum Command {
    Stop,
}

pub struct Started {
    pub sample_rate: f32,
    pub _channels: usize,
    pub sink: Sender<f32>,
}

pub fn build_audio_thread() -> (Receiver<Started>, Sender<Command>) {
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

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &Receiver<f32>)
where
    T: Sample + Send + Sync,
{
    for frame in output.chunks_mut(channels) {
        let Ok(value) = next_sample.recv() else {
            return;
        };
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
    events: Sender<Started>,
) -> Result<()>
where
    T: Sample + Send + Sync,
{
    let sample_rate = cfg.sample_rate.0 as f32;
    let channels = cfg.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on stream: {err}");

    let (tx, rx) = crossbeam_channel::bounded(cfg.sample_rate.0 as usize / 250);

    let stream = dev.build_output_stream(
        cfg,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| write_data(data, channels, &rx),
        err_fn,
    )?;
    stream.play()?;

    events.send(Started {
        sample_rate,
        _channels: channels,
        sink: tx,
    })?;

    match commands.recv() {
        Ok(Command::Stop) => Ok(()),
        Err(_) => Ok(()),
    }
}

pub fn build_midi_connection() -> Result<Receiver<Result<MidiMessage<'static>, FromBytesError>>> {
    let midi_in = MidiInput::new("PCMG Input")?;
    let in_ports = midi_in.ports();
    let Some(port) = in_ports.first() else {
        return Err(anyhow!("No midi port available"));
    };
    let (midi_tx, midi_rx) = crossbeam_channel::bounded(64);
    let _in_conn = midi_in
        .connect(
            port,
            "pcmg-input-port",
            move |_t, msg, _| {
                midi_tx
                    .send(MidiMessage::try_from(msg).map(|m| m.to_owned()))
                    .unwrap();
            },
            (),
        )
        .map_err(|e| anyhow!("Midi connction error: {e:?}"))?;
    Ok(midi_rx)
}
