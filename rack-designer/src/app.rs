use eframe::{
    App,
    Frame,
};
use egui::{
    CentralPanel,
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
use egui_file::State as FileDialogState;
use emath::{
    Pos2,
    Rect,
};
use rack::{
    container::sizing::ModuleSize,
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
}

impl RackDesigner {
    pub fn new() -> Self {
        Self {
            state: DesignerState::Empty,
        }
    }
}

impl App for RackDesigner {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.state = match std::mem::take(&mut self.state) {
            DesignerState::Empty => show_empty(ctx),
            DesignerState::New(state) => show_new(ctx, state),
            DesignerState::Load(state) => show_load(ctx, state),
            DesignerState::Save(state) => show_save(ctx, state),
            DesignerState::Edit(state) => show_edit(ctx, state),
        }
    }
}

fn show_edit(ctx: &Context, mut state: EditState) -> DesignerState {
    if let Some(mut adder) = state.widget_adder.take() {
        let closing = adder.show(ctx);
        if let false = closing {
            state.widget_adder = Some(adder);
        } else if let (true, Some(w)) = (closing, adder.widget) {
            state.module.visuals.push(adder.widgets.remove(&w).unwrap())
        }
    }

    if let Some(mut adder) = state.device_adder.take() {
        let (closing, selected) = adder.show(ctx);
        if !closing && !selected {
            state.device_adder = Some(adder);
        } else if selected && closing {
            state.module.devices.push(adder.devices[adder.device]);
        }
    }

    SidePanel::left("sidebar")
        .resizable(false)
        .min_width(256.)
        .show(ctx, |ui| {
            ScrollArea::vertical().id_source("widgets").show(ui, |ui| {
                ui.label("Widgets");
                if ui.button("Add").clicked() && state.widget_adder.is_none() {
                    // TODO: handle io errors
                    state.widget_adder = Some(WidgetAdder::new().unwrap())
                }
                for w in state.module.visuals.iter_mut() {
                    ui.separator();
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
                }
            });
            ui.separator();
            ScrollArea::vertical().id_source("devices").show(ui, |ui| {
                ui.label("Devices");
                if ui.button("Add").clicked() && state.device_adder.is_none() {
                    state.device_adder = Some(DeviceAdder::new())
                }
                for (di, dev) in state.module.devices.iter_mut().enumerate() {
                    ui.separator();
                    ui.label(format!("{di}: {}", dev.name()));
                    ui.indent((di, dev.name()), |ui| {
                        for (pi, param) in dev.params().iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{pi}: {}", param));
                                let label = state
                                    .module
                                    .connections
                                    .get(&(di, pi))
                                    .map_or("Connect", |c| &*state.module.visuals[*c].name);
                                ui.menu_button(label, |ui| {
                                    for (wi, w) in state.module.visuals.iter().enumerate() {
                                        if ui.button(&*w.name).clicked() {
                                            state.module.connections.insert((di, pi), wi);
                                        };
                                    }
                                })
                            });
                        }
                    });
                }
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
        paint_module_vidgets(ui, r.center(), &current.module.visuals);
    });
    next_state
}

fn labelled_drag_value(ui: &mut Ui, l: &str, v: &mut f32) {
    ui.horizontal(|ui| {
        ui.label(l);
        ui.add(DragValue::new(v));
    });
}

fn show_load(ctx: &Context, mut state: LoadState) -> DesignerState {
    state.dialog.open();
    state.dialog.show(ctx);

    match state.dialog.state() {
        FileDialogState::Open => DesignerState::Load(state),
        FileDialogState::Closed | FileDialogState::Cancelled => *state.previous,
        FileDialogState::Selected => {
            // TODO: error handling
            let s = std::fs::read_to_string(state.dialog.path().unwrap()).unwrap();
            let module: ModuleDescription = serde_yaml::from_str(&s).unwrap();
            let state = EditState::with_module(module);
            DesignerState::Edit(state)
        }
    }
}

fn show_save(ctx: &Context, mut state: SaveState) -> DesignerState {
    state.dialog.open();
    state.dialog.show(ctx);

    match state.dialog.state() {
        FileDialogState::Open => DesignerState::Save(state),
        FileDialogState::Closed | FileDialogState::Cancelled => DesignerState::Edit(state.previous),
        FileDialogState::Selected => {
            // TODO: error handling
            let module = state.previous.module.clone();
            let module = serde_yaml::to_string(&module).unwrap();
            std::fs::write(state.dialog.path().unwrap(), module).unwrap();
            DesignerState::Edit(state.previous)
        }
    }
}

fn show_empty(ctx: &Context) -> DesignerState {
    CentralPanel::default()
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label("No module loaded");
                    let load = ui.button("Load").clicked();
                    let new = ui.button("New").clicked();
                    if load {
                        DesignerState::Load(LoadState::new(DesignerState::Empty))
                    } else if new {
                        DesignerState::New(NewState::new(DesignerState::Empty))
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

fn paint_module_vidgets(ui: &mut Ui, center: Pos2, visuals: &[WidgetDescription]) {
    for visual in visuals {
        ui.put(
            Rect::from_center_size(center + visual.pos.to_vec2(), visual.size),
            visual,
        );
    }
}
