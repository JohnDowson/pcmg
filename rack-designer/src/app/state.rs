use rack::widget_description::ModuleDescription;

use emath::vec2;

use egui_file::FileDialog;

use rack::container::sizing::ModuleSize;

#[derive(Default)]
pub(crate) enum DesignerState {
    #[default]
    Empty,
    New(NewState),
    Load(LoadState),
    Edit(EditState),
}

pub(crate) struct NewState {
    pub(crate) previous: Box<DesignerState>,
    pub(crate) size: ModuleSize,
}

impl NewState {
    pub(crate) fn new(previous: DesignerState) -> Self {
        Self {
            previous: Box::new(previous),
            size: ModuleSize::U1,
        }
    }
}

pub(crate) struct LoadState {
    pub(crate) previous: Box<DesignerState>,
    pub(crate) dialog: FileDialog,
}

impl LoadState {
    pub(crate) fn new(previous: DesignerState) -> Self {
        Self {
            previous: Box::new(previous),
            dialog: FileDialog::open_file(None)
                .resizable(false)
                .default_size(vec2(4000., 4000.)),
        }
    }
}

#[derive(Clone)]
pub(crate) struct EditState {
    pub(crate) module: ModuleDescription,
}

impl EditState {
    pub(crate) fn new(size: ModuleSize) -> Self {
        Self {
            module: ModuleDescription {
                size,
                visuals: Default::default(),
                devices: Default::default(),
                connections: Default::default(),
            },
        }
    }
}
