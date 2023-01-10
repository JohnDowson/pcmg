use anyhow::Result;
use cpal::{traits::*, Sample, SampleFormat};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use eframe::egui::{self, CentralPanel, Checkbox, DragValue};
use pcmg::types::{
    filters::KrajeskiLadder, generators::Osc, FusedGenerator, Pipeline, PipelineSelector,
};
use std::marker::{Send, Sync};

pub enum Command {
    Stop,
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

fn build_audio_thread() -> (Receiver<Started>, Sender<Command>) {
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
    let (ev_rx, _cmd_tx) = build_audio_thread();

    let Started {
        sample_rate,
        _channels: _,
        sink,
    } = ev_rx.recv()?;

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 400.0)),
        ..Default::default()
    };

    let (gui, channel) = PcmGui::new();

    std::thread::spawn(move || {
        let mut oscs = FusedGenerator::new();
        let waveform = |p: f32| p.sin().asin();
        let osc = Osc::new(sample_rate, Box::new(waveform));
        oscs.push(osc);
        let mut pipeline = Pipeline::new(oscs, [1.0], KrajeskiLadder::new(sample_rate, 250.0, 0.5));

        let mut next_value = move || {
            match channel.try_recv() {
                Ok(e) => match e {
                    GuiEvent::FreqChanged(f) => {
                        pipeline.set_param(PipelineSelector::Osc((0, "freq"), f))
                    }
                    GuiEvent::CutoffChanged(f) => {
                        pipeline.set_param(PipelineSelector::Filter("cutoff", f))
                    }
                    _ => (),
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    return Err(anyhow::Error::from(TryRecvError::Disconnected))
                }
            }
            let sample = pipeline.sample();
            sink.send(Sample::from(&sample))?;
            Ok(())
        };

        while next_value().is_ok() {}
    });

    eframe::run_native("pcmg", options, Box::new(|_cc| Box::new(gui)));
    Ok(())
}

#[derive(Debug)]
pub enum GuiEvent {
    CutoffChanged(f32),
    ResonanceChanged(f32),
    FilterChanged(bool),
    FreqChanged(f32),
    LfoFreqChanged(f32),
    LfoChanged(bool),
}

struct PcmGui {
    resonance: f32,
    cutoff: f32,
    filter: bool,
    freq: f32,
    lfo_freq: f32,
    lfo: bool,
    channel: Sender<GuiEvent>,
}

impl PcmGui {
    fn new() -> (Self, Receiver<GuiEvent>) {
        let (tx, rx) = crossbeam_channel::bounded(128);
        let gui = Self {
            resonance: 0.0,
            cutoff: 0.0,
            filter: false,
            freq: 0.0,
            lfo_freq: 0.0,
            lfo: false,
            channel: tx,
        };
        (gui, rx)
    }
}

impl eframe::App for PcmGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("PCMG");
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            let cutoff = ui.add(
                                DragValue::new(&mut self.cutoff)
                                    .clamp_range(0.0..=1000.0)
                                    .speed(0.1),
                            );
                            ui.label("Cutoff");
                            if cutoff.changed() {
                                self.channel
                                    .send(GuiEvent::CutoffChanged(self.cutoff))
                                    .unwrap();
                            }
                        });
                        ui.vertical(|ui| {
                            let resonance = ui.add(
                                DragValue::new(&mut self.resonance)
                                    .clamp_range(0.0..=2.0)
                                    .speed(0.001),
                            );
                            ui.label("Resonance");

                            if resonance.changed() {
                                self.channel
                                    .send(GuiEvent::ResonanceChanged(self.resonance))
                                    .unwrap();
                            }
                        });
                    });
                    ui.horizontal(|ui| {
                        let filter = ui.add(Checkbox::new(&mut self.filter, "Filter"));
                        if filter.changed() {
                            self.channel
                                .send(GuiEvent::FilterChanged(self.filter))
                                .unwrap();
                        }
                    });
                });
                ui.vertical(|ui| {
                    let freq = ui.add(
                        DragValue::new(&mut self.freq)
                            .clamp_range(0.0..=1000.0)
                            .speed(0.1),
                    );
                    ui.label("Freq");
                    if freq.changed() {
                        self.channel.send(GuiEvent::FreqChanged(self.freq)).unwrap();
                    }
                });
                ui.vertical(|ui| {
                    let lfo_freq = ui.add(
                        DragValue::new(&mut self.lfo_freq)
                            .clamp_range(0.0..=100.0)
                            .speed(0.01),
                    );
                    ui.label("LFO freq");
                    let lfo = ui.add(Checkbox::new(&mut self.lfo, "LFO"));
                    if lfo_freq.changed() {
                        self.channel
                            .send(GuiEvent::LfoFreqChanged(self.lfo_freq))
                            .unwrap();
                    }
                    if lfo.changed() {
                        self.channel.send(GuiEvent::LfoChanged(self.lfo)).unwrap();
                    }
                });
            });
        });
    }
}
