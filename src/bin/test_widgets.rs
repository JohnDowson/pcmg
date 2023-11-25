use eframe::egui;
use pcmg::widgets::fader::Fader;

struct TemplateApp {
    fader: Fader,
    fader2: Fader,
}

impl TemplateApp {
    fn new() -> Self {
        Self {
            fader: Fader::new(0.0, (0.0, 1.0), 1.0),
            fader2: Fader::new(1.0, (0.0, 1.0), 1.0),
        }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(&mut self.fader);
                ui.add(&mut self.fader2);
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Widget test",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(TemplateApp::new())),
    )
}
