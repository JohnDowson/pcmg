#![feature(ptr_metadata)]

pub mod compiled_graph;
pub mod consts;
pub mod devices;
pub mod graph;
pub mod types;
#[cfg(target_arch = "wasm32")]
pub mod wasm_thread;
pub mod waves;
pub mod widgets;

use crate::{compiled_graph::compile, graph::UiMessage};
use anyhow::Result;
use cpal::{traits::*, SampleFormat, SizedSample, Stream};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use midir::MidiInput;
use std::collections::BTreeMap;
use wmidi::{MidiMessage, Note};

#[cfg(not(target_arch = "wasm32"))]
use std::thread::spawn;
#[cfg(target_arch = "wasm32")]
use wasm_thread::spawn;

pub struct NoteQueue {
    inner: Vec<(u64, Note)>,
}

impl NoteQueue {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn insert(&mut self, note: Note, time: u64) {
        if let Some((t, _)) = self.inner.iter_mut().find(|(_, n)| n == &note) {
            *t = time;
        } else {
            self.inner.push((time, note));
        }
    }

    pub fn remove(&mut self, note: Note) {
        if let Some(i) = self
            .inner
            .iter_mut()
            .enumerate()
            .find_map(|(i, (_, n))| (n == &note).then_some(i))
        {
            self.inner.remove(i);
        }
    }

    pub fn first(&self) -> Option<&Note> {
        self.inner.iter().min_by_key(|(t, _)| t).map(|(_, n)| n)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for NoteQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_sampler_thread(
    ui_rx: Receiver<UiMessage>,
    midi_rx: Receiver<(u64, MidiMessage<'static>)>,
    midi_ctl_tx: Sender<MidiCtlMsg>,
    sink: Sender<f32>,
) {
    let _ = spawn(move || {
        let mut pipeline = compile(&BTreeMap::default());
        let mut graph = Default::default();

        let mut notes = NoteQueue::new();
        let mut next_value = || {
            match ui_rx.try_recv() {
                Ok(msg) => match msg {
                    UiMessage::Rebuild(r) => {
                        graph = r;
                        pipeline = compile(&graph.2);
                    }
                    UiMessage::KnobChanged(nid, value) => pipeline.update_param(nid, value),
                    UiMessage::MidiPortChanged(n) => {
                        midi_ctl_tx.send(MidiCtlMsg::ChangePort(n)).unwrap()
                    }
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    return Err(anyhow::Error::from(TryRecvError::Disconnected))
                }
            }
            match midi_rx.try_recv() {
                Ok((t, m)) => match m {
                    MidiMessage::NoteOff(_, n, _) => {
                        notes.remove(n);
                        let f = if let Some(n) = notes.first() {
                            n.to_freq_f32()
                        } else {
                            0.0
                        };
                        for node in &graph.1 {
                            pipeline.update_param(*node, f)
                        }
                    }
                    MidiMessage::NoteOn(_, n, _) => {
                        notes.insert(n, t);
                        let f = n.to_freq_f32();
                        for node in &graph.1 {
                            pipeline.update_param(*node, f)
                        }
                    }
                    _ => (),
                },
                Err(TryRecvError::Empty) => (),
                Err(_) => todo!(),
            }
            let sample = pipeline.sample();
            sink.send(sample)?;
            Ok(())
        };

        while next_value().is_ok() {}
    });
}

pub fn build_audio_thread() -> (Stream, Sender<f32>) {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("no output device available");
    let supported_config = device
        .default_output_config()
        .expect("no output config available");
    let sample_format = supported_config.sample_format();
    let config = supported_config.into();

    match sample_format {
        SampleFormat::F32 => run::<f32>(&device, &config),
        f => panic!("Unsupported format {f:?}"),
    }
    .expect("Audio thread died")
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &Receiver<f32>)
where
    T: SizedSample + Send + Sync,
    T: From<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value = if let Ok(v) = next_sample.recv() {
            v
        } else {
            0.0
        };
        let value = T::from(value);
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

fn run<T>(dev: &cpal::Device, cfg: &cpal::StreamConfig) -> Result<(Stream, Sender<f32>)>
where
    T: SizedSample + Send + Sync + From<f32>,
{
    let sample_rate = cfg.sample_rate.0 as f32;
    let channels = cfg.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on stream: {err}");

    let (sink, rx) = crossbeam_channel::bounded(cfg.sample_rate.0 as usize / 250);

    let stream = dev.build_output_stream(
        cfg,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| write_data(data, channels, &rx),
        err_fn,
        None,
    )?;
    Ok((stream, sink))
}

pub type BuildMidiConnectionResult = Result<Vec<String>>;

pub enum MidiCtlMsg {
    ChangePort(usize),
}

pub fn build_midi_connection(
    midi_tx: Sender<(u64, MidiMessage<'static>)>,
    midi_ctl_rx: Receiver<MidiCtlMsg>,
    port_n: usize,
) -> BuildMidiConnectionResult {
    let midi_in = MidiInput::new("PCMG Input")?;

    let in_ports = midi_in.ports();
    let in_ports = in_ports
        .iter()
        .map(|p| midi_in.port_name(p).unwrap())
        .collect();

    let _ = spawn(move || {
        let mut port_n = port_n;
        loop {
            let midi_in = MidiInput::new("PCMG Input").unwrap();
            let in_ports = midi_in.ports();
            let Some(in_port) = in_ports.get(port_n) else {
                eprintln!("Port {port_n} isn't available");

                let MidiCtlMsg::ChangePort(n) = midi_ctl_rx.recv().unwrap();
                port_n = n;
                continue;
            };

            let tx = midi_tx.clone();
            let in_conn = match midi_in.connect(
                in_port,
                "pcmg-input-port",
                move |t, msg, _| {
                    let msg = MidiMessage::try_from(msg).map(|m| m.to_owned()).unwrap();
                    tx.send((t, msg)).unwrap();
                },
                (),
            ) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Midi connction error: {e:?}");
                    let MidiCtlMsg::ChangePort(n) = midi_ctl_rx.recv().unwrap();
                    port_n = n;
                    continue;
                }
            };

            if let Ok(MidiCtlMsg::ChangePort(n)) = midi_ctl_rx.recv() {
                in_conn.close();
                port_n = n;
            } else {
                eprintln!("Closing midi thread");
                return;
            }
        }
    });
    Ok(in_ports)
}
