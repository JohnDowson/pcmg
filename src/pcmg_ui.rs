use cpal::Stream;
use eframe::{
    egui::{
        CentralPanel,
        ComboBox,
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
use midir::MidiInputConnection;
use pcmg::build_midi_in;
use rack::{
    container::{
        Stack,
        StackResponse,
    },
    graph::modules::Module,
    widgets::scope::SampleQueue,
    STQueue,
};

use self::module_adder::ModuleAdder;

mod module_adder;

pub struct PcmgUi {
    ports: Vec<String>,
    port: Option<usize>,
    midi_conn: Option<MidiInputConnection<()>>,
    samples: SampleQueue,
    #[allow(dead_code)]
    stream: Stream,

    stack: Stack,
    adder: Option<ModuleAdder>,
}

impl PcmgUi {
    pub fn new(
        ui_tx: STQueue<StackResponse>,
        stream: Stream,
        ports: Vec<String>,
        midi_conn: Option<MidiInputConnection<()>>,
        samples: SampleQueue,
    ) -> Self {
        Self {
            ports,
            port: None,
            midi_conn,
            samples,
            stream,
            stack: Stack::new(ui_tx),
            adder: None,
        }
    }

    fn handle_midi_selector(&mut self, ui: &mut eframe::egui::Ui) {
        let port = self.port;
        let selected_text = if let Some(p) = self.port {
            &self.ports[p]
        } else {
            "None"
        };
        ComboBox::from_label("MIDI in")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for (i, port) in self.ports.iter().enumerate() {
                    ui.selectable_value(&mut self.port, Some(i), port);
                }
            });
        if port != self.port {
            if let Some(p) = self.port {
                let midi_evs = STQueue::new();
                let (_, mut conn) = build_midi_in(midi_evs.clone(), p).unwrap();
                std::mem::swap(&mut self.midi_conn, &mut conn);
                conn.map(|c| c.close());
                self.stack.events.put(StackResponse::MidiChange(midi_evs));
            } else {
                self.midi_conn.take().map(|c| c.close());
            }
        }
    }
}

impl App for PcmgUi {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        TopBottomPanel::top("top-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.handle_midi_selector(ui);

                #[cfg(target_arch = "wasm32")]
                {
                    use cpal::traits::StreamTrait;
                    if ui.button("Start sound").clicked() {
                        self.stream.play().unwrap();
                    }
                }
            });

            if ui.button("Add module").clicked() && self.adder.is_none() {
                self.adder = Some(ModuleAdder::new().unwrap());
            }
        });

        if let Some(a) = &mut self.adder {
            if a.show(ctx) {
                let m = a.selection;
                let mut m = self.adder.take().unwrap().modules.remove(m).1;

                for w in m.visuals.values_mut() {
                    w.position += (m.size.size() / 2.0) - (w.size / 2.0);
                }

                let m = Module::insert_from_description(&mut self.stack.graph, m);
                let added = self.stack.with_module(m).is_none();
                assert!(added);
            }
        }
        SidePanel::right("scope").show(ctx, |ui| {
            let sin: PlotPoints = self
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
                .include_x(44000.0 / 10.0)
                // .view_aspect(2.0)
                .show(ui, |plot_ui| plot_ui.line(line));

            ctx.request_repaint();
        });

        CentralPanel::default().show(ctx, |ui| {
            self.stack.show(ctx, ui);
        });
    }
}
