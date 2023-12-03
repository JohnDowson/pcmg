use anyhow::Result;
use pcmg::{
    build_audio,
    build_midi_in,
};
use pcmg_ui::PcmgUi;
use rack::{
    widgets::scope::SampleQueue,
    STQueue,
};

mod pcmg_ui;

fn main() -> Result<()> {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let midi_evs = STQueue::new();
    let ui_evs = STQueue::new();
    let samples = SampleQueue::new(44000 / 10);

    let (midi_ports, midi_conn) = build_midi_in(midi_evs.clone(), 0)?;

    let stream = build_audio(ui_evs.clone(), midi_evs.clone(), samples.clone());

    let app = PcmgUi::new(ui_evs, stream, midi_ports, midi_conn, samples);

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
