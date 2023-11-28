use app::WidgetDesigner;

mod app;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "widget-designer",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(WidgetDesigner::new())),
    )
}
