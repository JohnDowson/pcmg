use eframe::egui::{
    Context,
    Sense,
    Window,
};
use rack::widget_description::ModuleDescription;

pub struct ModuleAdder {
    pub selection: usize,
    pub modules: Vec<(String, ModuleDescription)>,
}

impl ModuleAdder {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut modules = Vec::new();
        for r in std::fs::read_dir("./prefab_modules")? {
            let f = r?;
            if f.file_type()?.is_file() {
                let s = std::fs::read_to_string(f.path())?;
                let m = serde_yaml::from_str(&s)?;
                modules.push((f.file_name().to_string_lossy().into_owned(), m));
            }
        }
        if modules.is_empty() {
            return Err(anyhow::anyhow!("No modules found!").into());
        }
        Ok(Self {
            selection: 0,
            modules,
        })
    }

    pub fn show(&mut self, ctx: &Context) -> bool {
        let mut closing = false;

        Window::new("Add module").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for (i, (filename, _)) in self.modules.iter().enumerate() {
                        ui.selectable_value(&mut self.selection, i, filename);
                    }
                    closing = ui.button("Add").clicked();
                });
                ui.vertical(|ui| {
                    let m = &self.modules[self.selection].1;
                    let (r, _) = ui.allocate_exact_size(m.size.size(), Sense::hover());
                    for w in m.visuals.values() {
                        w.preview(ui, r.center(), Default::default(), 0.0);
                    }
                });
            });
        });
        closing
    }
}
