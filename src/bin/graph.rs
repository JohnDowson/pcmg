use anyhow::Result;
use pcmg::{
    build_audio_thread, build_midi_connection, build_sampler_thread, graph::PcmgNodeGraph, Started,
};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
    use anyhow::anyhow;

    let (ev_rx, _cmd_tx) = build_audio_thread();

    let Started {
        sample_rate: _,
        _channels,
        sink,
    } = ev_rx.recv()?;

    let (midi_rx, midi_ctl_tx, midi_ports) = build_midi_connection(0)?;

    let (app, ui_rx) = PcmgNodeGraph::new(midi_ports);

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
    use pcmg::wasm_thread::spawn;

    console_error_panic_hook::set_once();

    wasm_bindgen_futures::spawn_local(async {
        // _ = spawn(|| {
        //     _ = spawn(|| panic!("thread in a thread"));
        // });

        _ = spawn(|| {
            let (ev_rx, _cmd_tx) = build_audio_thread();

            let Started {
                sample_rate: _,
                _channels,
                sink,
            } = ev_rx.recv().expect("Audio thread never started");

            let (midi_rx, midi_ctl_tx, midi_ports) =
                build_midi_connection(0).expect("Failed to build MIDI thread");

            let (app, ui_rx) = PcmgNodeGraph::new(midi_ports);

            build_sampler_thread(ui_rx, midi_rx, midi_ctl_tx, sink);

            let web_options = eframe::WebOptions::default();

            wasm_bindgen_futures::spawn_local(async {
                eframe::start_web("the_canvas", web_options, Box::new(|_cc| Box::new(app)))
                    .await
                    .expect("failed to start eframe");
            });
        });
    });
    Ok(())
}
