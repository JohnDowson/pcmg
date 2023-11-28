use app::ModuleDesigner;

mod app;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "module-designer",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(ModuleDesigner::new())),
    )
}
