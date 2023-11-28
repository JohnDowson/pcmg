use std::{fs, path::PathBuf};

use eframe::{
    egui::{
        vec2, CentralPanel, ComboBox, Context, PointerButton, Sense, TextEdit, TopBottomPanel,
        Vec2, Window,
    },
    epaint::{Color32, Pos2, Rect},
};
use egui_file::FileDialog;

use rack::{
    container::sizing::SlotSize,
    error_window,
    widget_description::{ModuleDescription, Wid},
};

use self::adder::WidgetAdder;

mod adder;

pub struct ModuleDesigner {
    module: ModuleDescription,
    widget_adder: Option<WidgetAdder>,
    saver: FileDialog,
    opener: FileDialog,
    next_wid: u16,
}

impl ModuleDesigner {
    pub fn new() -> Self {
        Self {
            module: ModuleDescription {
                size: SlotSize::U1,
                widgets: Default::default(),
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
            .for_each(|(_, w)| w.pos -= offset.to_vec2());
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
            .for_each(|(_, w)| w.pos += offset.to_vec2());
        self.module = module;
        Ok(())
    }
}

impl eframe::App for ModuleDesigner {
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

            if let Some(pt) = ctx.pointer_latest_pos() {
                let mpt = pt - r.min;
                ui.put(
                    Rect::from_min_size(pt, vec2(256., 32.)),
                    TextEdit::singleline(&mut format!("Screen: {pt:?}, mod: {mpt:?}"))
                        .interactive(false),
                );
            }

            for (id, mut w) in std::mem::take(&mut self.module.widgets) {
                let resp = ui.add(&w);

                if resp.clicked_by(PointerButton::Secondary) {
                    let Vec2 { x: xs, y: ys } = resp.rect.size() * 0.125;

                    let wp = w.pos + resp.drag_delta();
                    let Vec2 { x: xp, y: yp } = wp - r.min;
                    let x = (xp / xs).round() * xs;
                    let y = (yp / ys).round() * ys;
                    w.pos = r.min + vec2(x, y);
                }

                if resp.dragged_by(PointerButton::Primary) {
                    w.pos += resp.drag_delta();
                }

                if !resp.clicked_by(PointerButton::Middle) {
                    self.module.widgets.insert(id, w);
                }
            }

            if mr.clicked_by(PointerButton::Secondary) && self.widget_adder.is_none() {
                let pos = ctx.pointer_interact_pos().unwrap_or(r.max);
                let wa = WidgetAdder::new(pos);
                if let Ok(wa) = wa {
                    self.widget_adder = Some(wa);
                } else {
                    error_window("Could not load widget prefabs", ctx)
                }
            }

            if let Some(mut wa) = self.widget_adder.take() {
                wa.show(ctx);

                if let WidgetAdder {
                    pos,
                    widget: Some(uuid),
                    closing: true,
                    mut widgets,
                } = wa
                {
                    let mut widget = widgets.remove(&uuid).unwrap();
                    widget.pos = pos;
                    let wid = Wid(self.next_wid);
                    self.next_wid += 1;
                    self.module.widgets.insert(wid, widget);
                } else {
                    self.widget_adder = Some(wa);
                }
            }
        });
    }
}
