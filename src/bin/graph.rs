use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use cpal::Sample;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use pcmg::{
    build_audio_thread, build_midi_connection,
    compiled_graph::compile,
    graph::{PcmgNodeGraph, UiMessage},
    Started,
};
use wmidi::MidiMessage;

fn main() -> Result<()> {
    let (app, ui_rx) = PcmgNodeGraph::new();
    let (ev_rx, _cmd_tx) = build_audio_thread();

    let Started {
        sample_rate: _,
        _channels,
        sink,
    } = ev_rx.recv()?;

    let midi_rx = build_midi_connection()?;

    run_sampler(ui_rx, midi_rx, sink);

    eframe::run_native(
        "Egui node graph example",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app)),
    );

    Ok(())
}

fn run_sampler(
    ui_rx: Receiver<UiMessage>,
    midi_rx: Receiver<Result<MidiMessage<'static>, wmidi::FromBytesError>>,
    sink: Sender<f32>,
) {
    std::thread::spawn(move || {
        let mut pipeline = compile(&BTreeMap::default());

        let mut notes = BTreeSet::new();
        let mut next_value = || {
            match ui_rx.try_recv() {
                Ok(msg) => match msg {
                    UiMessage::Rebuild(r) => {
                        let src = &r.1;
                        pipeline = compile(src);
                    }
                    UiMessage::KnobChanged(nid, value) => pipeline.update_param(nid, value),
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    return Err(anyhow::Error::from(TryRecvError::Disconnected))
                }
            }
            match midi_rx.try_recv() {
                Ok(Ok(m)) => match m {
                    MidiMessage::NoteOff(_, n, _) => {
                        notes.remove(&n);
                        // if notes.is_empty() {
                        //     pipeline.let_go();
                        // }
                    }
                    MidiMessage::NoteOn(_, n, _) => {
                        notes.insert(n);
                        // pipeline.trigger();
                        // let f = n.to_freq_f32();
                        // pipeline.set_param(PipelineSelector::Osc(None, GenSel::Freq, f));
                    }
                    _ => (),
                },
                Ok(Err(_)) => (),
                Err(_) => (),
            }
            let sample = pipeline.sample();
            sink.send(Sample::from(&sample))?;
            Ok(())
        };

        while next_value().is_ok() {}
    });
}
