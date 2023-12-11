use anyhow::Result;
use pcmg_ui::PcmgUi;
use rack::module_description::ModuleDescription;
use rack_loaders::{
    assetloader::ModulePrefab,
    AssetLoader,
};

mod pcmg_ui;

fn main() -> Result<()> {
    let mut loader = AssetLoader::<ModuleDescription>::new(
        #[cfg(target_arch = "wasm32")]
        "pcmg_modules",
    )
    .unwrap();
    if let Err(e) = loader.load_embeds::<ModulePrefab>() {
        log::warn!("{e}");
    }

    let app = PcmgUi::new(loader);

    #[cfg(not(target_arch = "wasm32"))]
    eframe::run_native(
        "Egui node graph example",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app)),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    #[cfg(target_arch = "wasm32")]
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
