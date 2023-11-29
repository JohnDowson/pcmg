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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stack = Stack::new();

    let mut sid = 0;
    for r in std::fs::read_dir("./prefab_modules")? {
        let f = r?;
        if f.file_type()?.is_file() {
            let module = load_module(f.path())?;
            stack.with_module(Slot::from_description(Sid(sid), module));
            sid += 1;
        }
    }

    let app = RackLayout::new(vec![stack]);

    eframe::run_native(
        "rack-designer",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app)),
    )?;
    Ok(())
}

fn load_module(path: PathBuf) -> Result<ModuleDescription, Box<dyn std::error::Error>> {
    let s = fs::read_to_string(path)?;
    let module: ModuleDescription = serde_yaml::from_str(&s)?;

    Ok(module)
}
