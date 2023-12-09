use app::RackDesigner;

use rack::visuals::templates::WidgetTemplate;
use rack_loaders::AssetLoader;

mod app;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let widget_loader = AssetLoader::<WidgetTemplate>::new().unwrap();

    eframe::run_native(
        "rack-designer",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(RackDesigner::new(widget_loader))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();

    let widget_loader: AssetLoader<WidgetTemplate> = AssetLoader::new("pcmg_widgets").unwrap();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "egui-canvas",
                eframe::WebOptions::default(),
                Box::new(|_cc| Box::new(RackDesigner::new(widget_loader))),
            )
            .await
            .expect("failed to start eframe");
    });
}
