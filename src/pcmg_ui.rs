use cpal::Stream;
use eframe::{
    egui::{
        CentralPanel,
        ComboBox,
        Context,
        TopBottomPanel,
    },
    App,
    Frame,
};
use midir::MidiInputConnection;
use pcmg::build_midi_in;
use rack::{
    container::{
        Stack,
        StackResponse,
    },
    STQueue,
};

pub struct PcmgUi {
    ui_tx: STQueue<StackResponse>,

    ports: Vec<String>,
    port: Option<usize>,
    midi_conn: Option<MidiInputConnection<()>>,

    #[allow(dead_code)]
    stream: Stream,

    stack: Stack,
}
impl PcmgUi {
    pub fn new(
        ui_tx: STQueue<StackResponse>,
        stream: Stream,
        ports: Vec<String>,
        midi_conn: Option<MidiInputConnection<()>>,
    ) -> Self {
        Self {
            ui_tx,
            ports,
            port: None,
            midi_conn,
            stream,
            stack: Stack::new(),
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
                self.ui_tx.put(StackResponse::MidiChange(midi_evs));
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
                    if ui.add(egui::Button::new("Start sound")).clicked() {
                        self.stream.play().unwrap();
                    }
                }
            });

            if ui.button("Add module").clicked() {
                //
            }
        });

        CentralPanel::default().show(ctx, |ui| {
            if let Some(msg) = self.stack.show(ctx, ui) {
                self.ui_tx.put(msg);
            }
        });
    }
}
