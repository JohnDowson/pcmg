use app::RackDesigner;

mod app;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    eframe::run_native(
        "rack-designer",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(RackDesigner::new())),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "egui-canvas",
                eframe::WebOptions::default(),
                Box::new(|_cc| Box::new(RackDesigner::new())),
            )
            .await
            .expect("failed to start eframe");
    });
}
