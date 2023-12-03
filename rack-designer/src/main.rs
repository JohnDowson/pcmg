use app::RackDesigner;

mod app;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "rack-designer",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(RackDesigner::new())),
    )
}
