use std::collections::BTreeMap;

use anyhow::Result;
use cpal::Sample;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use pcmg::{
    build_audio_thread, build_midi_connection,
    compiled_graph::compile,
    graph::{PcmgNodeGraph, UiMessage},
    MidiCtlMsg, NoteQueue, Started,
};
use wmidi::MidiMessage;

fn main() -> Result<()> {
    let (ev_rx, _cmd_tx) = build_audio_thread();

    let Started {
        sample_rate: _,
        _channels,
        sink,
    } = ev_rx.recv()?;

    let (midi_rx, midi_ctl_tx, midi_ports) = build_midi_connection(0)?;

    let (app, ui_rx) = PcmgNodeGraph::new(midi_ports);

    run_sampler(ui_rx, midi_rx, midi_ctl_tx, sink);

    eframe::run_native(
        "Egui node graph example",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app)),
    );

    Ok(())
}

fn run_sampler(
    ui_rx: Receiver<UiMessage>,
    midi_rx: Receiver<(u64, MidiMessage<'static>)>,
    midi_ctl_tx: Sender<MidiCtlMsg>,
    sink: Sender<f32>,
) {
    std::thread::spawn(move || {
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
            sink.send(Sample::from(&sample))?;
            Ok(())
        };

        while next_value().is_ok() {}
    });
}
