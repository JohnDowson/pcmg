use std::{
    fs,
    path::PathBuf,
};

use egui::{
    CentralPanel,
    Context,
};
use rack::{
    container::{
        sizing::SlotSize,
        Slot,
        Stack,
    },
    widget_description::{
        ModuleDescription,
        Sid,
    },
};

pub struct RackLayout {
    slots: Vec<Stack>,
}

impl RackLayout {
    pub fn new(slots: Vec<Stack>) -> Self {
        Self { slots }
    }
}

impl eframe::App for RackLayout {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            for stack in &mut self.slots {
                stack.show(ui)
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let mut stack = Stack::new();

    stack.with_module(Slot::from_description(
        Sid(0),
        load_module("./pmodule.yml".into()).unwrap(),
    ));
    // stack.with_module(Slot::empty(SlotSize::U1));
    // stack.with_module(Slot::empty(SlotSize::U2));
    // stack.with_module(Slot::empty(SlotSize::U2));
    stack.with_module(Slot::empty(SlotSize::U2));
    // stack.with_module(Slot::empty(SlotSize::U1));
    // stack.with_module(Slot::empty(SlotSize::U1));

    let app = RackLayout::new(vec![stack]);

    eframe::run_native(
        "rack-designer",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app)),
    )
}

fn load_module(path: PathBuf) -> Result<ModuleDescription, Box<dyn std::error::Error>> {
    let s = fs::read_to_string(path)?;
    let module: ModuleDescription = serde_yaml::from_str(&s)?;

    Ok(module)
}
