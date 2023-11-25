use anyhow::Result;
use cpal::traits::StreamTrait as _;
use pcmg::{build_audio_thread, build_midi_connection, build_sampler_thread, graph::PcmgNodeGraph};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
    use anyhow::anyhow;

    let (midi_ctl_tx, midi_ctl_rx) = crossbeam_channel::bounded(64);
    let (midi_tx, midi_rx) = crossbeam_channel::bounded(64);
    let (ui_tx, ui_rx) = crossbeam_channel::unbounded();

    let (stream, sink) = build_audio_thread();

    let midi_ports =
        build_midi_connection(midi_tx, midi_ctl_rx, 0).expect("Failed to build MIDI thread");

    stream.play().unwrap();
    let app = PcmgNodeGraph::new(ui_tx, midi_ports, stream);

    build_sampler_thread(ui_rx, midi_rx, midi_ctl_tx, sink);
    eframe::run_native(
        "Egui node graph example",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app)),
    )
    .map_err(|e| anyhow!("{e}"))
}

#[cfg(target_arch = "wasm32")]
fn main() -> Result<()> {
    console_error_panic_hook::set_once();

    let (midi_ctl_tx, midi_ctl_rx) = crossbeam_channel::bounded(64);
    let (midi_tx, midi_rx) = crossbeam_channel::bounded(64);
    let (ui_tx, ui_rx) = crossbeam_channel::unbounded();

    let (stream, sink) = build_audio_thread();

    let midi_ports =
        build_midi_connection(midi_tx, midi_ctl_rx, 0).expect("Failed to build MIDI thread");

    let midi_ports = vec!["fake".into()];
    stream.play().unwrap();
    let app = PcmgNodeGraph::new(ui_tx, midi_ports, stream);

    build_sampler_thread(ui_rx, midi_rx, midi_ctl_tx, sink);

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web("egui-canvas", web_options, Box::new(|_cc| Box::new(app)))
            .await
            .expect("failed to start eframe");
    });

    Ok(())
}
