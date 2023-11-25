use anyhow::Result;
use pcmg::{build_audio, build_midi_in, graph::PcmgNodeGraph};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
    use anyhow::anyhow;

    let (midi_ctl_tx, midi_ctl_rx) = crossbeam_channel::bounded(64);
    let (midi_tx, midi_rx) = crossbeam_channel::bounded(64);
    let (ui_tx, ui_rx) = crossbeam_channel::unbounded();

    let (stream, sink) = build_audio_thread();

    let midi_ports = build_midi_in(midi_tx, midi_ctl_rx, 0).expect("Failed to build MIDI thread");

    stream.play().unwrap();
    let app = PcmgNodeGraph::new(ui_tx, midi_ports, stream);

    build_audio(ui_rx, midi_rx, midi_ctl_tx, sink);
    eframe::run_native(
        "Egui node graph example",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app)),
    )
    .map_err(|e| anyhow!("{e}"))
}

#[cfg(target_arch = "wasm32")]
fn main() -> Result<()> {
    use pcmg::STQueue;
    use web_sys::console;

    console_error_panic_hook::set_once();

    let midi_evs = STQueue::new();
    let ui_evs = STQueue::new();

    console::log_1(&"Building initial midi conn".into());
    let (midi_ports, midi_conn) = build_midi_in(midi_evs.clone(), 0)?;

    console::log_1(&"Building audio".into());
    let stream = build_audio(ui_evs.clone(), midi_evs.clone());

    console::log_1(&"Building app".into());
    let app = PcmgNodeGraph::new(ui_evs, stream, midi_ports, midi_conn);

    console::log_1(&"Starting app".into());
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web("egui-canvas", web_options, Box::new(|_cc| Box::new(app)))
            .await
            .expect("failed to start eframe");
    });

    Ok(())
}
