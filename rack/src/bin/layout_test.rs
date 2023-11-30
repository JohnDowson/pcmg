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
        module::Module,
        Stack,
    },
    widget_description::ModuleDescription,
};

pub struct RackLayout {
    stack: Stack,
}

impl RackLayout {
    pub fn new(stack: Stack) -> Self {
        Self { stack }
    }
}

impl eframe::App for RackLayout {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| self.stack.show(ctx, ui));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stack = Stack::new();

    for r in std::fs::read_dir("./prefab_modules")? {
        let f = r?;
        if f.file_type()?.is_file() {
            let module = load_module(f.path())?;
            let id = Module::insert_from_description(&mut stack.graph, module);
            if stack.with_module(id).is_some() {
                eprintln!("Layout fail");
            }
        }
    }

    let app = RackLayout::new(stack);

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
