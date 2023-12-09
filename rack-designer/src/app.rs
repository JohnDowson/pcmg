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
use futures::channel::mpsc;
use rack::{
    container::sizing::ModuleSize,
    devices::description::Param,
    pos_drag_value,
    two_drag_value,
    visuals::templates::WidgetTemplate,
    widget_description::{
        ModuleDescription,
        WidgetKind,
    },
};
#[cfg(target_arch = "wasm32")]
use rack_loaders::saveloaders::save_to_url;
use rack_loaders::{
    saveloaders::{
        loader,
        saver,
    },
    AssetLoader,
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
    loading_chan: Option<mpsc::Receiver<Option<ModuleDescription>>>,
    widget_loader: AssetLoader<WidgetTemplate>,
}

impl RackDesigner {
    pub fn new(widget_loader: AssetLoader<WidgetTemplate>) -> Self {
        Self {
            state: DesignerState::Empty,
            loading_chan: None,
            widget_loader,
        }
    }

    pub fn with_module(&mut self, module: ModuleDescription) {
        self.state = DesignerState::Edit(EditState::with_module(module))
    }

    pub(crate) fn with_widget(&mut self, widget: WidgetTemplate) {
        self.state = DesignerState::WidgetEditor(WidgetEditorState::with_widget(widget))
    }
}

impl App for RackDesigner {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.widget_loader.drive();
        self.state = match std::mem::take(&mut self.state) {
            DesignerState::Empty => show_empty(ctx),
            DesignerState::New(state) => show_new(ctx, state),
            DesignerState::Load(state) => {
                let rx = loader();
                self.loading_chan = Some(rx);
                DesignerState::Loading(state)
            }
            DesignerState::Loading(state) => {
                match self.loading_chan.as_mut().map(|rx| rx.try_next()) {
                    Some(Ok(Some(Some(module)))) => {
                        DesignerState::Edit(EditState::with_module(module))
                    }
                    Some(Ok(Some(None))) => *state.previous,
                    Some(Err(_)) => DesignerState::Loading(state),
                    Some(Ok(None)) => panic!("Closed"),
                    None => panic!("None"),
                }
            }
            DesignerState::Save(state) => {
                saver(state.previous.module.clone());
                DesignerState::Edit(state.previous)
            }
            DesignerState::Edit(state) => show_edit(ctx, state, &self.widget_loader),
            DesignerState::WidgetEditor(state) => state.show(ctx),
        }
    }
}

fn show_edit(
    ctx: &Context,
    mut state: EditState,
    loader: &AssetLoader<WidgetTemplate>,
) -> DesignerState {
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
                .devices
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
                widgets_editor(ui, &mut state, loader);
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
                let back = ui.button("Exit to menu").clicked();
                let new = ui.button("New").clicked();
                let save = ui.button("Save").clicked();
                #[cfg(target_arch = "wasm32")]
                {
                    let export = ui.button("Get share URL").clicked();
                    if export {
                        save_to_url("pcmg_module", state.module.clone());
                    }
                }
                let load = ui.button("Load").clicked();
                if back {
                    DesignerState::Empty
                } else if new {
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

fn widgets_editor(ui: &mut Ui, state: &mut EditState, loader: &AssetLoader<WidgetTemplate>) {
    ui.label("Widgets");
    if ui.button("Load widgets").clicked() {
        loader.load()
    }
    if ui.button("Add").clicked() && state.widget_adder.is_none() {
        state.widget_adder = Some(WidgetAdder::new(loader.assets()))
    }
    for (i, w) in state.module.visuals.iter_mut() {
        ui.separator();
        CollapsingHeader::new(w.name.clone())
            .id_source(*i)
            .show(ui, |ui| {
                ui.text_edit_singleline(&mut w.name);
                pos_drag_value(ui, "Position (center)", &mut w.position);
                match &mut w.kind {
                    WidgetKind::Knob(k) => {
                        two_drag_value(
                            ui,
                            "Angle range",
                            "Start",
                            "End",
                            &mut k.angle_range.start,
                            &mut k.angle_range.end,
                        );
                        two_drag_value(
                            ui,
                            "Value range",
                            "start",
                            "end",
                            &mut k.value_range.start,
                            &mut k.value_range.end,
                        );

                        labelled_drag_value(ui, "Speed", &mut k.speed);
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

fn paint_module_widgets(ui: &mut Ui, center: Pos2, visuals: &BTreeMap<usize, WidgetTemplate>) {
    visuals.values().for_each(|visual| {
        visual.preview(
            ui,
            center + visual.position.to_vec2(),
            Default::default(),
            0.0,
        )
    });
}
