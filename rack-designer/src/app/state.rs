use rack::widget_description::ModuleDescription;

use rack::container::sizing::ModuleSize;
use uuid::Uuid;

use self::widget_editor::WidgetEditorState;

use super::adder::{
    DeviceAdder,
    WidgetAdder,
};

pub mod widget_editor;

#[derive(Default)]
pub enum DesignerState {
    #[default]
    Empty,
    WidgetEditor(WidgetEditorState),
    New(NewState),
    Load(LoadState),
    Loading(LoadState),
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
}

impl LoadState {
    pub fn new(previous: DesignerState) -> Self {
        Self {
            previous: Box::new(previous),
        }
    }
}

pub struct SaveState {
    pub previous: EditState,
}

impl SaveState {
    pub fn new(previous: EditState) -> Self {
        Self { previous }
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
                uuid: Uuid::new_v4(),
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
