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
};
use std::collections::{BTreeSet, VecDeque};
use wmidi::MidiMessage;

fn main() -> Result<()> {
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
