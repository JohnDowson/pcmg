use std::collections::BTreeMap;

use eframe::{
    App,
    Frame,
};
use egui::{
    CentralPanel,
    CollapsingHeader,
    Color32,
    ComboBox,
    Context,
    DragValue,
    Painter,
    Rounding,
    ScrollArea,
    Sense,
    SidePanel,
    Stroke,
    TopBottomPanel,
    Ui,
};
use emath::{
    Pos2,
    Rect,
};
use rack::{
    container::sizing::ModuleSize,
    devices::description::Param,
    pos_drag_value,
    two_drag_value,
    widget_description::{
        ModuleDescription,
        WidgetDescription,
        WidgetKind,
    },
};

use self::{
    adder::{
        DeviceAdder,
        WidgetAdder,
    },
    state::{
        widget_editor::WidgetEditorState,
        DesignerState,
        EditState,
        LoadState,
        NewState,
        SaveState,
    },
};

mod adder;
mod state;

pub struct RackDesigner {
    state: DesignerState,
    #[cfg(target_arch = "wasm32")]
    loading_chan: Option<futures::channel::mpsc::Receiver<Option<EditState>>>,
}

impl RackDesigner {
    pub fn new() -> Self {
        Self {
            state: DesignerState::Empty,
            #[cfg(target_arch = "wasm32")]
            loading_chan: None,
        }
    }
}

impl App for RackDesigner {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.state = match std::mem::take(&mut self.state) {
            DesignerState::Empty => show_empty(ctx),
            DesignerState::New(state) => show_new(ctx, state),
            #[cfg(not(target_arch = "wasm32"))]
            DesignerState::Load(state) => show_load(state),
            #[cfg(target_arch = "wasm32")]
            DesignerState::Load(state) => {
                let (loading, rx) = show_load(state);
                self.loading_chan = Some(rx);
                loading
            }
            #[cfg(target_arch = "wasm32")]
            DesignerState::Loading(state) => {
                match self.loading_chan.as_mut().map(|rx| rx.try_next()) {
                    Some(Ok(Some(Some(estate)))) => DesignerState::Edit(estate),
                    Some(Ok(Some(None))) => *state.previous,
                    Some(Err(_)) => DesignerState::Loading(state),
                    Some(Ok(None)) => panic!("Closed"),
                    None => panic!("None"),
                }
            }
            DesignerState::Save(state) => show_save(state),
            DesignerState::Edit(state) => show_edit(ctx, state),
            DesignerState::WidgetEditor(state) => state.show(ctx),
        }
    }
}

fn show_edit(ctx: &Context, mut state: EditState) -> DesignerState {
    if let Some(mut adder) = state.widget_adder.take() {
        let closing = adder.show(ctx);
        if let false = closing {
            state.widget_adder = Some(adder);
        } else if let (true, Some(w)) = (closing, adder.widget) {
            let k = state
                .module
                .visuals
                .last_key_value()
                .map(|(k, _)| k + 1)
                .unwrap_or_default();

            state
                .module
                .visuals
                .insert(k, adder.widgets.remove(&w).unwrap());
        }
    }

    if let Some(mut adder) = state.device_adder.take() {
        let (closing, selected) = adder.show(ctx);
        if !closing && !selected {
            state.device_adder = Some(adder);
        } else if selected && closing {
            let k = state
                .module
                .visuals
                .last_key_value()
                .map(|(k, _)| k + 1)
                .unwrap_or_default();
            state.module.devices.insert(k, adder.devices[adder.device]);
        }
    }

    SidePanel::left("sidebar")
        .resizable(false)
        .min_width(256.)
        .show(ctx, |ui| {
            ScrollArea::vertical().id_source("widgets").show(ui, |ui| {
                widgets_editor(ui, &mut state);
            });
            ui.separator();
            ScrollArea::vertical().id_source("devices").show(ui, |ui| {
                devices_editor(ui, &mut state);
            });
        });

    let current = state.clone();
    let next_state = TopBottomPanel::top("toolbar")
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let new = ui.button("New").clicked();
                let save = ui.button("Save").clicked();
                let load = ui.button("Load").clicked();
                if new {
                    DesignerState::New(NewState::new(DesignerState::Edit(state)))
                } else if save {
                    DesignerState::Save(SaveState::new(state))
                } else if load {
                    DesignerState::Load(LoadState::new(DesignerState::Edit(state)))
                } else {
                    DesignerState::Edit(state)
                }
            })
            .inner
        })
        .inner;

    CentralPanel::default().show(ctx, |ui| {
        let r = ui.available_rect_before_wrap();
        paint_module_bg(ui.painter(), r.center(), current.module.size);
        paint_module_widgets(ui, r.center(), &current.module.visuals);
    });
    next_state
}

fn devices_editor(ui: &mut Ui, state: &mut EditState) {
    ui.label("Devices");
    if ui.button("Add").clicked() && state.device_adder.is_none() {
        state.device_adder = Some(DeviceAdder::new())
    }
    for (di, dev) in state.module.devices.iter_mut() {
        ui.separator();
        ui.label(format!("{di}: {}", dev.name()));
        ui.indent((di, dev.name()), |ui| {
            for (pi, param) in dev.params().iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{pi}: {}", param));
                    let label = state
                        .module
                        .connections
                        .get(&(*di, pi))
                        .map_or("Connect", |c| &*state.module.visuals[c].name);
                    ui.menu_button(label, |ui| {
                        for (wi, w) in state.module.visuals.iter() {
                            let show = match (param, w.kind) {
                                (Param::In(_), WidgetKind::Knob(_)) => true,
                                (Param::In(_), WidgetKind::Port) => true,
                                (Param::Out(_), WidgetKind::Knob(_)) => true,
                                (Param::Out(_), WidgetKind::Port) => true,
                            };
                            if show && ui.button(&*w.name).clicked() {
                                state.module.connections.insert((*di, pi), *wi);
                            };
                        }
                    })
                });
            }
        });
    }
}

fn widgets_editor(ui: &mut Ui, state: &mut EditState) {
    ui.label("Widgets");
    if ui.button("Add").clicked() && state.widget_adder.is_none() {
        // TODO: handle io errors
        state.widget_adder = Some(WidgetAdder::new().unwrap())
    }
    for (i, w) in state.module.visuals.iter_mut() {
        ui.separator();
        CollapsingHeader::new(w.name.clone())
            .id_source(*i)
            .show(ui, |ui| {
                ui.text_edit_singleline(&mut w.name);
                pos_drag_value(ui, "Position (center)", &mut w.pos);
                match &mut w.kind {
                    WidgetKind::Knob(k) => {
                        two_drag_value(
                            ui,
                            "Value range",
                            "Start",
                            "End",
                            &mut k.value_range.0,
                            &mut k.value_range.1,
                        );
                        two_drag_value(
                            ui,
                            "Angle range",
                            "Start",
                            "End",
                            &mut k.angle_range.0,
                            &mut k.angle_range.1,
                        );

                        labelled_drag_value(ui, "Speed", &mut k.speed);
                        labelled_drag_value(ui, "Default position", &mut k.default_pos)
                    }
                    WidgetKind::Port => {}
                }
            });
    }
}

fn labelled_drag_value(ui: &mut Ui, l: &str, v: &mut f32) {
    ui.horizontal(|ui| {
        ui.label(l);
        ui.add(DragValue::new(v));
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn show_load(state: LoadState) -> DesignerState {
    let file = rfd::FileDialog::new().set_directory(".").pick_file();
    match file {
        None => *state.previous,
        Some(file) => {
            // TODO: error handling
            let s = std::fs::read_to_string(file).unwrap();
            let module: ModuleDescription = serde_yaml::from_str(&s).unwrap();
            let state = EditState::with_module(module);
            DesignerState::Edit(state)
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn show_load(
    state: LoadState,
) -> (
    DesignerState,
    futures::channel::mpsc::Receiver<Option<EditState>>,
) {
    let (mut tx, rx) = futures::channel::mpsc::channel(1);
    wasm_bindgen_futures::spawn_local(async move {
        let file = rfd::AsyncFileDialog::new()
            .set_directory(".")
            .pick_file()
            .await;
        _ = match file {
            None => tx.try_send(None),
            Some(file) => {
                let file = file.read().await;
                // TODO: error handling
                let module: ModuleDescription = serde_yaml::from_slice(&file).unwrap();
                let state = EditState::with_module(module);
                tx.try_send(Some(state))
            }
        };
    });
    (DesignerState::Loading(state), rx)
}

fn show_save(state: SaveState) -> DesignerState {
    #[cfg(target_arch = "wasm32")]
    {
        let module = state.previous.module.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let file = rfd::AsyncFileDialog::new()
                .set_directory(".")
                .save_file()
                .await;
            match file {
                None => (),
                Some(file) => {
                    let module = serde_yaml::to_string(&module).unwrap();
                    // TODO: error handling
                    file.write(module.as_bytes()).await.unwrap();
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let file = rfd::FileDialog::new().set_directory(".").save_file();
        match file {
            None => (),
            Some(file) => {
                let module = state.previous.module.clone();
                let module = serde_yaml::to_string(&module).unwrap();
                // TODO: error handling
                std::fs::write(file, module.as_bytes()).unwrap();
            }
        }
    }
    DesignerState::Edit(state.previous)
}

fn show_empty(ctx: &Context) -> DesignerState {
    CentralPanel::default()
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label("No module loaded");
                    let load = ui.button("Load").clicked();
                    let new = ui.button("New").clicked();
                    let widget = ui.button("Widget").clicked();
                    if load {
                        DesignerState::Load(LoadState::new(DesignerState::Empty))
                    } else if new {
                        DesignerState::New(NewState::new(DesignerState::Empty))
                    } else if widget {
                        DesignerState::WidgetEditor(WidgetEditorState::new())
                    } else {
                        DesignerState::Empty
                    }
                })
                .inner
            })
            .inner
        })
        .inner
}

fn show_new(ctx: &Context, mut state: NewState) -> DesignerState {
    CentralPanel::default()
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.vertical_centered(|ui| {
                    ComboBox::from_label("Select module size")
                        .selected_text(state.size.to_string())
                        .show_ui(ui, |ui| {
                            for size in ModuleSize::all() {
                                ui.selectable_value(&mut state.size, size, size.to_string());
                            }
                        });
                    let (r, p) = ui.allocate_painter(state.size.size(), Sense::hover());
                    paint_module_bg(&p, r.rect.center(), state.size);

                    let create = ui.button("Create").clicked();
                    let cancel = ui.button("Cancel").clicked();
                    if create {
                        DesignerState::Edit(EditState::new(state.size))
                    } else if cancel {
                        *state.previous
                    } else {
                        DesignerState::New(state)
                    }
                })
                .inner
            })
            .inner
        })
        .inner
}

fn paint_module_bg(p: &Painter, center: Pos2, size: ModuleSize) {
    let r = Rect::from_center_size(center, size.size());
    p.rect_stroke(
        r,
        Rounding::ZERO,
        Stroke {
            width: 2.0,
            color: Color32::from_rgb(60, 140, 0),
        },
    );
}

fn paint_module_widgets(ui: &mut Ui, center: Pos2, visuals: &BTreeMap<usize, WidgetDescription>) {
    visuals.values().for_each(|visual| {
        ui.put(
            Rect::from_center_size(center + visual.pos.to_vec2(), visual.size),
            visual,
        );
    });
}
