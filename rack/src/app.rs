use std::{fs, path::PathBuf};

use eframe::{
    egui::{self, CentralPanel, ComboBox, Context, PointerButton, Sense, TopBottomPanel},
    epaint::{Color32, Pos2, Rect},
};
use egui::Window;
use egui_file::FileDialog;
use rack::{
    container::sizing::SlotSize,
    widget_description::{ModuleDescription, Wid, WidgetDescription},
};

use self::adder::WidgetAdder;

mod adder;

pub struct RackDesigner {
    module: ModuleDescription,
    widget_adder: Option<WidgetAdder>,
    saver: FileDialog,
    opener: FileDialog,
    next_wid: u16,
}

impl RackDesigner {
    pub fn new() -> Self {
        Self {
            module: ModuleDescription {
                size: SlotSize::U1,
                widgets: Vec::new(),
            },
            widget_adder: None,
            saver: FileDialog::save_file(None),
            opener: FileDialog::open_file(None),
            next_wid: 0,
        }
    }

    fn save_widgets(&self, offset: Pos2, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let mut ws = self.module.clone();
        ws.widgets
            .iter_mut()
            .for_each(|w| w.pos -= offset.to_vec2());
        let s = serde_yaml::to_string(&ws)?;
        fs::write(path, s)?;
        Ok(())
    }

    fn load_widgets(
        &mut self,
        offset: Pos2,
        path: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let s = fs::read_to_string(path)?;
        let mut module: ModuleDescription = serde_yaml::from_str(&s)?;
        module
            .widgets
            .iter_mut()
            .for_each(|w| w.pos += offset.to_vec2());
        self.module = module;
        Ok(())
    }
}

impl eframe::App for RackDesigner {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("Toolbar").show(ctx, |ui| {
            ComboBox::from_label("Size")
                .selected_text(self.module.size.to_string())
                .show_ui(ui, |ui| {
                    for size in SlotSize::all() {
                        ui.selectable_value(&mut self.module.size, size, size.to_string());
                    }
                });

            if ui.button("Save").clicked() && matches!(self.saver.state(), egui_file::State::Closed)
            {
                self.saver.open();
            }

            if ui.button("Open").clicked()
                && matches!(self.opener.state(), egui_file::State::Closed)
            {
                self.opener.open();
            }
        });

        CentralPanel::default().show(ctx, |ui| {
            let r = Rect::from_min_size(ui.next_widget_position(), self.module.size.size());

            match self.saver.show(ctx).state() {
                egui_file::State::Open => {}
                egui_file::State::Closed => {}
                egui_file::State::Cancelled => {}
                egui_file::State::Selected => {
                    let _ = self.save_widgets(r.min, self.saver.path().unwrap());
                }
            }

            match self.opener.show(ctx).state() {
                egui_file::State::Open => {}
                egui_file::State::Closed => {}
                egui_file::State::Cancelled => {}
                egui_file::State::Selected => {
                    let _ = self.load_widgets(r.min, self.opener.path().unwrap());
                }
            }

            let mr = ui.allocate_rect(r, Sense::click());

            let p = ui.painter();
            p.debug_rect(
                r,
                Color32::from_rgb(100, 140, 80),
                self.module.size.to_string(),
            );

            for mut w in std::mem::take(&mut self.module.widgets) {
                if !ui.add(&mut w).clicked_by(PointerButton::Middle) {
                    self.module.widgets.push(w)
                }
            }

            if mr.clicked_by(PointerButton::Secondary) && self.widget_adder.is_none() {
                let pos = ctx.pointer_interact_pos().unwrap_or(r.max);
                let mut wa = WidgetAdder::new(pos);
                if let Ok(prefabs) = prefabs("./prefabs.yaml".into()) {
                    wa.with_prefabs(prefabs);
                }
                self.widget_adder = Some(wa);
            }

            if let Some(mut wa) = self.widget_adder.take() {
                Window::new("New")
                    .resizable(false)
                    .show(ctx, |ui| ui.add(&mut wa));
                if let WidgetAdder {
                    pos,
                    mut prefab,
                    closing: true,
                    prefabs: _,
                } = wa
                {
                    prefab.pos = pos;
                    prefab.wid = Wid(self.next_wid);
                    self.next_wid += 1;
                    self.module.widgets.push(prefab);
                } else {
                    self.widget_adder = Some(wa);
                }
            }
        });
    }
}

fn prefabs(path: PathBuf) -> Result<Vec<WidgetDescription>, Box<dyn std::error::Error>> {
    let s = fs::read_to_string(path)?;
    let prefabs = serde_yaml::from_str(&s)?;
    Ok(prefabs)
}
