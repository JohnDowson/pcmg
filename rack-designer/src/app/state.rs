use rack::widget_description::ModuleDescription;

use egui_file::FileDialog;

use rack::container::sizing::ModuleSize;

use super::adder::{
    DeviceAdder,
    WidgetAdder,
};

#[derive(Default)]
pub enum DesignerState {
    #[default]
    Empty,
    New(NewState),
    Load(LoadState),
    Save(SaveState),
    Edit(EditState),
}

pub struct NewState {
    pub previous: Box<DesignerState>,
    pub size: ModuleSize,
}

impl NewState {
    pub fn new(previous: DesignerState) -> Self {
        Self {
            previous: Box::new(previous),
            size: ModuleSize::U1,
        }
    }
}

pub struct LoadState {
    pub previous: Box<DesignerState>,
    pub dialog: FileDialog,
}

impl LoadState {
    pub fn new(previous: DesignerState) -> Self {
        Self {
            previous: Box::new(previous),
            dialog: FileDialog::open_file(None).resizable(false),
        }
    }
}

pub struct SaveState {
    pub previous: EditState,
    pub dialog: FileDialog,
}

impl SaveState {
    pub fn new(previous: EditState) -> Self {
        Self {
            previous,
            dialog: FileDialog::save_file(None).resizable(false),
        }
    }
}

#[derive(Clone)]
pub struct EditState {
    pub widget_adder: Option<WidgetAdder>,
    pub device_adder: Option<DeviceAdder>,
    pub module: ModuleDescription,
}

impl EditState {
    pub fn new(size: ModuleSize) -> Self {
        Self {
            widget_adder: None,
            device_adder: None,
            module: ModuleDescription {
                size,
                visuals: Default::default(),
                devices: Default::default(),
                connections: Default::default(),
            },
        }
    }

    pub fn with_module(module: ModuleDescription) -> EditState {
        Self {
            module,
            widget_adder: None,
            device_adder: None,
        }
    }
}
