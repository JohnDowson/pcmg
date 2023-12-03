use eframe::{
    App,
    Frame,
};
use egui::{
    CentralPanel,
    Color32,
    ComboBox,
    Context,
    Rounding,
    Sense,
    SidePanel,
    Stroke,
    TopBottomPanel,
    Ui,
};
use egui_file::State as FileDialogState;
use emath::Rect;
use rack::{
    container::sizing::ModuleSize,
    widget_description::ModuleDescription,
};

use self::state::{
    DesignerState,
    EditState,
    LoadState,
    NewState,
};

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
            DesignerState::Edit(state) => show_edit(ctx, state),
        }
    }
}

fn show_edit(ctx: &Context, state: EditState) -> DesignerState {
    let current = state.clone();
    let next_state = TopBottomPanel::top("toolbar")
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let new = ui.button("New").clicked();
                let save = ui.button("Save").clicked();
                let load = ui.button("Load").clicked();
                if new {
                    DesignerState::New(NewState::new(DesignerState::Edit(current)))
                } else if save {
                    DesignerState::Empty
                } else if load {
                    DesignerState::Load(LoadState::new(DesignerState::Edit(current)))
                } else {
                    DesignerState::Edit(current)
                }
            })
            .inner
        })
        .inner;

    SidePanel::left("sidebar")
        .resizable(false)
        .min_width(256.)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("widgets go here");
            })
        });

    CentralPanel::default().show(ctx, |ui| {
        paint_module_bg(ui, state.module.size);
    });
    next_state
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
            let state = EditState { module };
            DesignerState::Edit(state)
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
                    paint_module_bg(ui, state.size);

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

fn paint_module_bg(ui: &mut Ui, size: ModuleSize) {
    let (r, p) = ui.allocate_painter(size.size(), Sense::hover());
    p.rect_stroke(
        Rect::from_center_size(r.rect.center(), size.size()),
        Rounding::ZERO,
        Stroke {
            width: 2.0,
            color: Color32::from_rgb(60, 140, 0),
        },
    );
}
