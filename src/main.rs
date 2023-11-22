use anyhow::Result;
use cpal::Sample;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use eframe::egui::{self, CentralPanel};
use pcmg::{
    build_audio_thread, build_midi_connection,
    types::{
        filters::KrajeskiLadder,
        generators::{FmOsc, Osc, SquarePulse},
        GenSel, LfoSel, Pipeline, PipelineSelector, ADSR,
    },
    widgets::{filter_group, lfo_knob, master_knob, osc_group, KnobGroup},
    Started,
};
use std::collections::{BTreeSet, VecDeque};
use wmidi::MidiMessage;

fn main() -> Result<()> {
    let (ev_rx, _cmd_tx) = build_audio_thread();

    let Started {
        sample_rate,
        _channels: _,
        sink,
    } = ev_rx.recv()?;

    let (midi_rx, _, _) = build_midi_connection(0)?;

    let (tx, rx) = crossbeam_channel::unbounded();

    let sine = |p: f32| p.sin();
    let lfo = Osc::new(sample_rate, sine);

    let adsr = ADSR::new(sample_rate, 1.0, 1.0, 0.5, 1.0, 0.3, 0.1);

    let mut pipeline = Pipeline::new(lfo, adsr);
    let mut groups = vec![vec![
        lfo_knob(LfoSel::Freq, 0.0..=10.0),
        lfo_knob(LfoSel::Depth, 0.0..=1.0),
        master_knob(),
    ]];

    let osc = SquarePulse::new(sample_rate);
    let (n, params) = pipeline.add_osc(osc, 1.0);
    groups.push(osc_group(n, params));

    let osc = Osc::new(sample_rate, sine);
    let (n, params) = pipeline.add_osc(osc, 1.0);
    groups.push(osc_group(n, params));

    let saw = |f: f32| f.tan().atan();
    let osc = Osc::new(sample_rate, saw);
    let (n, params) = pipeline.add_osc(osc, 1.0);
    groups.push(osc_group(n, params));

    let osc = FmOsc::new(sample_rate);
    let (n, params) = pipeline.add_osc(osc, 1.0);
    groups.push(osc_group(n, params));

    let filter = KrajeskiLadder::new(sample_rate, 0.0, 0.0);
    let (n, params) = pipeline.add_filter(filter);
    groups.push(filter_group(n, params));

    let filter = KrajeskiLadder::new(sample_rate, 0.0, 0.0);
    let (n, params) = pipeline.add_filter(filter);
    groups.push(filter_group(n, params));

    let (gui, channel) = PcmGui::new(KnobGroup::new(groups), rx);

    let mut notes = BTreeSet::new();

    std::thread::spawn(move || {
        let mut hold = false;
        let mut next_value = move || {
            match channel.try_recv() {
                Ok(e) => match e {
                    GuiEvent::ParamChanged(param) => pipeline.set_param(param),
                    GuiEvent::HoldToggled(new_hold) => {
                        hold = new_hold;
                        if !hold {
                            pipeline.let_go()
                        }
                    }
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    return Err(anyhow::Error::from(TryRecvError::Disconnected))
                }
            }
            if !hold {
                match midi_rx.try_recv() {
                    Ok((_, m)) => match m {
                        MidiMessage::NoteOff(_, n, _) => {
                            notes.remove(&n);
                            if notes.is_empty() {
                                pipeline.let_go();
                            }
                        }
                        MidiMessage::NoteOn(_, n, _) => {
                            notes.insert(n);
                            pipeline.trigger();
                            let f = n.to_freq_f32();
                            pipeline.set_param(PipelineSelector::Osc(None, GenSel::Freq, f));
                        }
                        _ => (),
                    },
                    Err(_) => todo!(),
                }
            } else {
                notes.clear();
                pipeline.hold();
            }
            let sample = pipeline.sample();
            sink.send(Sample::from(&sample))?;
            tx.send(sample)?;
            Ok(())
        };

        while next_value().is_ok() {}
    });

    eframe::run_native(
        "pcmg",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(gui)),
    );
    Ok(())
}

enum GuiEvent {
    ParamChanged(PipelineSelector),
    HoldToggled(bool),
}

struct PcmGui {
    channel: Sender<GuiEvent>,
    knobs: KnobGroup<f32, PipelineSelector>,
    hold: bool,
    distortion: bool,
    samples_recv: Receiver<f32>,
    samples: VecDeque<f32>,
}

impl PcmGui {
    fn new(
        knobs: KnobGroup<f32, PipelineSelector>,
        samples_recv: Receiver<f32>,
    ) -> (Self, Receiver<GuiEvent>) {
        let (tx, rx) = crossbeam_channel::bounded(128);

        let gui = Self {
            channel: tx,
            knobs,
            hold: false,
            distortion: false,
            samples_recv,
            samples: VecDeque::new(),
        };
        (gui, rx)
    }
}

impl eframe::App for PcmGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("PCMG");

            while let Ok(sample) = self.samples_recv.try_recv() {
                self.samples.push_back(sample);
                if self.samples.len() == 1024 * 2 {
                    self.samples.pop_front();
                }
            }

            use egui::plot::{Line, Plot, PlotPoints};
            let sin: PlotPoints = self
                .samples
                .iter()
                .copied()
                .enumerate()
                .map(|(i, s)| [i as f64, s as f64])
                .collect();
            let line = Line::new(sin);
            Plot::new("Waveform")
                .view_aspect(2.0)
                .show(ui, |plot_ui| plot_ui.line(line));

            ctx.request_repaint();
            ui.add(&mut self.knobs);
            for change in self.knobs.changes() {
                self.channel.send(GuiEvent::ParamChanged(change)).unwrap();
            }
            ui.horizontal(|ui| {
                let hold = ui.checkbox(&mut self.hold, "Hold");
                let dist = ui.checkbox(&mut self.distortion, "Distortion");
                if hold.changed() {
                    self.channel.send(GuiEvent::HoldToggled(self.hold)).unwrap()
                }
                if dist.changed() {
                    self.channel
                        .send(GuiEvent::ParamChanged(PipelineSelector::Distortion(
                            self.distortion,
                        )))
                        .unwrap()
                }
            })
        });
    }
}
