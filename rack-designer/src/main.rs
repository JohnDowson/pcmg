use app::RackDesigner;

use rack::visuals::templates::WidgetTemplate;
use rack_loaders::{
    assetloader::WidgetPrefab,
    AssetLoader,
};
mod app;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let mut widget_loader = AssetLoader::<WidgetTemplate>::new().unwrap();
    widget_loader.load_embeds::<WidgetPrefab>().unwrap();

    eframe::run_native(
        "rack-designer",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(RackDesigner::new(widget_loader))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rack_loaders::saveloaders::load_from_url;

    console_log::init_with_level(log::Level::Debug).unwrap();

    log::info!("Loading prefabs");
    let mut widget_loader: AssetLoader<WidgetTemplate> = match AssetLoader::new("pcmg_widgets") {
        Ok(loader) => loader,
        Err(e) => {
            log::error!("{e:?}");
            panic!();
        }
    };
    widget_loader.load_embeds::<WidgetPrefab>()?;

    let mut app = RackDesigner::new(widget_loader);

    log::info!("Trying to load url share string");
    if let Some(module) = load_from_url("pcmg_module") {
        app.with_module(module)
    }

    if let Some(widget) = load_from_url("pcmg_widget") {
        app.with_widget(widget)
    }

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "egui-canvas",
                eframe::WebOptions::default(),
                Box::new(|_cc| Box::new(app)),
            )
            .await
            .expect("failed to start eframe");
    });
    Ok(())
}
