use anyhow::{anyhow, Result};
use cpal::{traits::*, Sample, SampleFormat};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use eframe::egui::{self, CentralPanel};
use midir::MidiInput;
use pcmg::{
    types::{
        filters::KrajeskiLadder,
        generators::{Osc, SquarePulse},
        GenSel, LfoSel, Pipeline, PipelineSelector, ADSR,
    },
    widgets::{filter_group, lfo_knob, master_knob, osc_group, KnobGroup},
};
use std::{
    collections::{BTreeSet, VecDeque},
    io::stdin,
    marker::{Send, Sync},
};
use wmidi::MidiMessage;

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
        let Ok(value) = next_sample.recv() else { return };
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

    let midi_in = MidiInput::new("PCMG Input")?;
    let in_ports = midi_in.ports();
    let ports = in_ports
        .iter()
        .enumerate()
        .map(|(i, p)| Ok((i, midi_in.port_name(p)?)))
        .collect::<Result<Vec<_>>>()?;

    let mut _in_conn;
    let mut input = String::new();
    println!("Select port number:");
    for (i, name) in ports {
        println!("{i}: {name}");
    }
    let (midi_tx, midi_rx) = crossbeam_channel::bounded(64);
    loop {
        stdin().read_line(&mut input)?;
        match input.trim().parse::<usize>() {
            Ok(n) => {
                if let Some(port) = in_ports.get(n) {
                    _in_conn = midi_in
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
                    break;
                }
            }
            Err(_) => (),
        }
        input.clear()
    }

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
                    Ok(Ok(m)) => match m {
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
                    Ok(Err(_)) => (),
                    Err(_) => (),
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

    eframe::run_native("pcmg", options, Box::new(|_cc| Box::new(gui)));
    Ok(())
}

enum GuiEvent {
    ParamChanged(PipelineSelector),
    HoldToggled(bool),
}

struct PcmGui {
    channel: Sender<GuiEvent>,
    knobs: KnobGroup<f32>,
    hold: bool,
    distortion: bool,
    samples_recv: Receiver<f32>,
    samples: VecDeque<f32>,
}

impl PcmGui {
    fn new(knobs: KnobGroup<f32>, samples_recv: Receiver<f32>) -> (Self, Receiver<GuiEvent>) {
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
