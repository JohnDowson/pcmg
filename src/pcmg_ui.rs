use std::time::Duration;

use cpal::{
    traits::{
        DeviceTrait,
        StreamTrait,
    },
    Device,
    Stream,
};
use eframe::{
    egui::{
        CentralPanel,
        Context,
        SidePanel,
        TopBottomPanel,
    },
    App,
    Frame,
};
use egui_plot::{
    Line,
    Plot,
    PlotPoints,
};
use midir::{
    MidiInputConnection,
    MidiInputPort,
};
use pcmg::{
    build_audio,
    build_midi_in,
    enumerate_midi_inputs,
    enumerate_outputs,
};
use rack::{
    container::Stack,
    graph::modules::Module,
    module_description::ModuleDescription,
    widgets::scope::SampleQueue,
    STQueue,
};
use rack_loaders::AssetLoader;

use self::module_adder::ModuleAdder;

mod module_adder;

struct Started {
    _midi_conn: Option<MidiInputConnection<()>>,
    samples: SampleQueue,
    sample_rate: f32,

    _stream: Stream,

    stack: Stack,
    adder: Option<ModuleAdder>,
    load_string: String,
}

struct PreStart {
    midi_ports: Vec<(String, MidiInputPort)>,
    selected_port: Option<usize>,
    audio_outputs: Vec<Device>,
    selected_output: Option<usize>,
}

enum PcmgUiState {
    PreStart(PreStart),
    Started(Started),
}

impl Default for PcmgUiState {
    fn default() -> Self {
        Self::PreStart(PreStart {
            midi_ports: Vec::new(),
            selected_port: None,
            audio_outputs: Vec::new(),
            selected_output: None,
        })
    }
}

pub struct PcmgUi {
    state: PcmgUiState,
    loader: AssetLoader<ModuleDescription>,
}

impl PcmgUi {
    pub fn new(loader: AssetLoader<ModuleDescription>) -> Self {
        Self {
            state: PcmgUiState::PreStart(PreStart {
                audio_outputs: enumerate_outputs(),
                selected_port: None,
                midi_ports: enumerate_midi_inputs(),
                selected_output: None,
            }),

            loader,
        }
    }
}

fn update_pre_start(ctx: &Context, mut state: PreStart) -> PcmgUiState {
    let next = CentralPanel::default()
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.label("MIDI input");
                    let label = state
                        .selected_port
                        .map(|s| &*state.midi_ports[s].0)
                        .unwrap_or("None");
                    ui.menu_button(label, |ui| {
                        for (i, (name, _)) in state.midi_ports.iter().enumerate() {
                            if ui.button(name).clicked() {
                                state.selected_port = Some(i);
                            }
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.label("Audio output");
                    let output_names: Vec<_> = state
                        .audio_outputs
                        .iter()
                        .map(|o| o.name().unwrap())
                        .collect();
                    let label = state
                        .selected_output
                        .map(|s| state.audio_outputs[s].name().unwrap())
                        .unwrap_or_else(|| "None".into());
                    ui.menu_button(label, |ui| {
                        for (i, name) in output_names.iter().enumerate() {
                            if ui.button(name).clicked() {
                                state.selected_output = Some(i);
                            }
                        }
                    });
                });

                let start = ui
                    .add_enabled_ui(state.selected_output.is_some(), |ui| ui.button("Start"))
                    .inner;

                if start.enabled() && start.clicked() {
                    let ui_evs = STQueue::new();
                    let midi_evs = STQueue::new();

                    let midi_conn = state.selected_port.and_then(|p| {
                        //
                        let (_, p) = state.midi_ports.remove(p);

                        build_midi_in(midi_evs.clone(), p)
                    });

                    let samples = SampleQueue::new(44100 / 10);
                    let (sample_rate, stream) = build_audio(
                        state.audio_outputs.remove(state.selected_output.unwrap()),
                        ui_evs.clone(),
                        midi_evs,
                        samples.clone(),
                    );
                    samples.set_period(sample_rate as _);

                    stream.play().unwrap();

                    PcmgUiState::Started(Started {
                        _midi_conn: midi_conn,
                        samples,
                        sample_rate,
                        _stream: stream,
                        stack: Stack::new(ui_evs),
                        adder: None,
                        load_string: String::new(),
                    })
                } else {
                    PcmgUiState::PreStart(state)
                }
            })
            .inner
        })
        .inner;
    next
}

fn update_started(
    ctx: &Context,
    mut state: Started,
    loader: &mut AssetLoader<ModuleDescription>,
) -> PcmgUiState {
    TopBottomPanel::top("top-bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Add module").clicked() && state.adder.is_none() {
                state.adder = Some(ModuleAdder::new(loader.assets()));
            }

            ui.label("Module share string");
            ui.text_edit_singleline(&mut state.load_string);
            if ui.button("Load").clicked() {
                loader.load_b64(&state.load_string);
                state.load_string.clear();
            }
        })
    });

    if let Some(a) = &mut state.adder {
        if a.show(ctx) {
            let m = a.selection.unwrap();
            let mut m = state.adder.take().unwrap().modules.remove(&m).unwrap();

            for w in m.visuals.values_mut() {
                w.position += (m.size.size() / 2.0) - (w.size / 2.0);
            }

            let m = Module::insert_from_description(&mut state.stack.graph, m);
            let added = state.stack.with_module(m).is_none();
            assert!(added);
        }
    }
    SidePanel::right("scope").show(ctx, |ui| {
        let sin: PlotPoints = state
            .samples
            .get()
            .iter()
            .copied()
            .enumerate()
            .map(|(i, s)| [i as f64, s as f64])
            .collect();
        let line = Line::new(sin);
        Plot::new("Waveform")
            .include_y(2.0)
            .include_y(-2.0)
            .include_x(0.0)
            .include_x(state.sample_rate / 10.0)
            // .view_aspect(2.0)
            .show(ui, |plot_ui| plot_ui.line(line));

        ctx.request_repaint_after(Duration::from_secs_f32(1. / 60.));
    });

    CentralPanel::default().show(ctx, |ui| {
        state.stack.show(ctx, ui);
    });

    PcmgUiState::Started(state)
}

impl App for PcmgUi {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.state = match std::mem::take(&mut self.state) {
            PcmgUiState::PreStart(state) => update_pre_start(ctx, state),
            PcmgUiState::Started(state) => update_started(ctx, state, &mut self.loader),
        };
    }
}
