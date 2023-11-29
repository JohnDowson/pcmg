pub mod compiled_graph;
pub mod consts;
pub mod devices;
pub mod graph;
pub mod types;
pub mod waves;

use crate::{
    compiled_graph::compile,
    graph::UiMessage,
};
use anyhow::Result;
use cpal::{
    traits::*,
    SampleFormat,
    Stream,
};
use midir::{
    MidiInput,
    MidiInputConnection,
};
use rack::widgets::scope::SampleQueue;
use std::{
    collections::{
        BTreeMap,
        VecDeque,
    },
    sync::Arc,
};
use wmidi::{
    MidiMessage,
    Note,
};

pub struct STQueue<T> {
    inner: Arc<eframe::epaint::mutex::Mutex<VecDeque<T>>>,
}

impl<T> Clone for STQueue<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> STQueue<T> {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn put(&self, msg: T) {
        self.inner.lock().push_front(msg)
    }

    pub fn get(&self) -> Option<T> {
        self.inner.lock().pop_back()
    }
}

impl<T> Default for STQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

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

pub fn build_audio(
    ui_evs: STQueue<UiMessage>,
    mut midi_evs: STQueue<(u64, MidiMessage<'static>)>,
    samples: SampleQueue,
) -> Stream {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("no output device available");
    let supported_config = device
        .default_output_config()
        .expect("no output config available");
    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();

    let stream = match sample_format {
        SampleFormat::F32 => {
            let channels = config.channels as usize;

            let err_fn = |err| eprintln!("an error occurred on stream: {err}");

            let mut pipeline = compile(&BTreeMap::default());
            let mut graph = Default::default();

            let mut notes = NoteQueue::new();
            let mut next_value = move || {
                if let Some(msg) = ui_evs.get() {
                    match msg {
                        UiMessage::Rebuild(r) => {
                            graph = r;
                            pipeline = compile(&graph.2);
                        }
                        UiMessage::KnobChanged(nid, value) => pipeline.update_param(nid, value),
                        UiMessage::MidiPortChanged(evs) => midi_evs = evs,
                    }
                }
                if let Some((t, m)) = midi_evs.get() {
                    match m {
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
                    }
                }
                let sample = pipeline.sample();
                samples.put(sample);
                sample
            };
            device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _| {
                        for frame in data.chunks_mut(channels) {
                            let value = next_value();
                            for sample in frame.iter_mut() {
                                *sample = value;
                            }
                        }
                    },
                    err_fn,
                    None,
                )
                .expect("Failed to build output stream")
        }
        f => panic!("Unsupported format {f:?}"),
    };
    stream.play().unwrap();
    stream
}

pub type BuildMidiConnectionResult = Result<(Vec<String>, Option<MidiInputConnection<()>>)>;

pub fn build_midi_in(
    midi_evs: STQueue<(u64, MidiMessage<'static>)>,
    port_n: usize,
) -> BuildMidiConnectionResult {
    let midi_in = MidiInput::new("PCMG Input")?;
    let in_ports = midi_in.ports();
    let in_ports_names = in_ports
        .iter()
        .map(|p| midi_in.port_name(p).unwrap())
        .collect();

    let Some(in_port) = in_ports.get(port_n) else {
        return Ok((in_ports_names, None));
    };

    let in_conn = midi_in
        .connect(
            in_port,
            "pcmg-input-port",
            move |t, msg, _| {
                let msg = MidiMessage::try_from(msg).map(|m| m.to_owned()).unwrap();
                midi_evs.put((t, msg));
            },
            (),
        )
        .ok();

    Ok((in_ports_names, in_conn))
}
