use anyhow::Result;
use pcmg::{build_audio, build_midi_in, graph::PcmgNodeGraph};

use pcmg::STQueue;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
    let midi_evs = STQueue::new();
    let ui_evs = STQueue::new();

    let (midi_ports, midi_conn) = build_midi_in(midi_evs.clone(), 0)?;

    let stream = build_audio(ui_evs.clone(), midi_evs.clone());

    let app = PcmgNodeGraph::new(ui_evs, stream, midi_ports, midi_conn);

    eframe::run_native(
        "Egui node graph example",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app)),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}

#[cfg(target_arch = "wasm32")]
fn main() -> Result<()> {
    console_error_panic_hook::set_once();

    let midi_evs = STQueue::new();
    let ui_evs = STQueue::new();

    let (midi_ports, midi_conn) = build_midi_in(midi_evs.clone(), 0)?;

    let stream = build_audio(ui_evs.clone(), midi_evs.clone());

    let app = PcmgNodeGraph::new(ui_evs, stream, midi_ports, midi_conn);

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web("egui-canvas", web_options, Box::new(|_cc| Box::new(app)))
            .await
            .expect("failed to start eframe");
    });

    Ok(())
}
