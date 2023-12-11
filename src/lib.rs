pub mod consts;
#[allow(dead_code)]
mod types;
pub mod waves;

use cpal::{
    traits::*,
    Device,
    SampleFormat,
    Stream,
};
use midir::{
    MidiInput,
    MidiInputConnection,
    MidiInputPort,
};
use rack::{
    container::StackResponse,
    graph::compiled::compile,
    widgets::scope::SampleQueue,
    STQueue,
};
use wmidi::{
    MidiMessage,
    Note,
};

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

pub fn enumerate_outputs() -> Vec<Device> {
    let host = cpal::default_host();
    host.output_devices().unwrap().collect()
}

pub fn build_audio(
    device: Device,
    ui_evs: STQueue<StackResponse>,
    mut midi_evs: STQueue<(u64, MidiMessage<'static>)>,
    samples: SampleQueue,
) -> Stream {
    let supported_config = device
        .default_output_config()
        .expect("no output config available");
    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();

    let stream = match sample_format {
        SampleFormat::F32 => {
            let channels = config.channels as usize;

            let err_fn = |err| eprintln!("an error occurred on stream: {err}");

            let mut pipeline = compile(&Default::default());
            let mut graph = Default::default();

            let mut notes = NoteQueue::new();
            let mut next_value = move || {
                if let Some(msg) = ui_evs.get() {
                    match msg {
                        StackResponse::Rebuild(r) => {
                            graph = r;
                            pipeline = compile(&graph);
                        }
                        StackResponse::ControlChange(nid, value) => {
                            if let Some(id) = graph.dev_map.get(&nid) {
                                pipeline.update_param(*id, value);
                            }
                        }
                        StackResponse::MidiChange(evs) => {
                            println!("Midi channel changed");
                            midi_evs = evs
                        }
                    }
                }
                if let Some((t, m)) = midi_evs.get() {
                    println!("Midi event recv'd");
                    match m {
                        MidiMessage::NoteOff(_, n, _) => {
                            notes.remove(n);
                            let f = if let Some(n) = notes.first() {
                                n.to_freq_f32()
                            } else {
                                0.0
                            };
                            for (_nid, pid @ (_, pi)) in &graph.midis {
                                let f = if *pi == 1 { 0.0 } else { f };
                                pipeline.update_param(*pid, f)
                            }
                        }
                        MidiMessage::NoteOn(_, n, _) => {
                            notes.insert(n, t);
                            let f = n.to_freq_f32();
                            for (_nid, pid @ (_, pi)) in &graph.midis {
                                let f = if *pi == 1 { 1.0 } else { f };
                                pipeline.update_param(*pid, f)
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

pub fn enumerate_midi_inputs() -> Vec<(String, MidiInputPort)> {
    if let Ok(midi_in) = MidiInput::new("PCMG Input") {
        midi_in
            .ports()
            .into_iter()
            .map(|p| (midi_in.port_name(&p).unwrap(), p))
            .collect()
    } else {
        Vec::new()
    }
}

pub fn build_midi_in(
    midi_evs: STQueue<(u64, MidiMessage<'static>)>,
    port: MidiInputPort,
) -> Option<MidiInputConnection<()>> {
    let midi_in = MidiInput::new("PCMG Input").ok()?;

    midi_in
        .connect(
            &port,
            "pcmg-input-port",
            move |t, msg, _| {
                let msg = MidiMessage::try_from(msg).map(|m| m.to_owned()).unwrap();
                midi_evs.put((t, msg));
            },
            (),
        )
        .ok()
}
