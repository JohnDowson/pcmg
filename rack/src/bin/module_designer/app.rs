use std::{
    collections::BTreeMap,
    fs,
    path::Path,
};

use eframe::{
    egui::{
        vec2,
        CentralPanel,
        ComboBox,
        Context,
        PointerButton,
        Sense,
        TopBottomPanel,
        Vec2,
    },
    epaint::{
        Color32,
        Pos2,
        Rect,
    },
};
use egui_file::FileDialog;

use emath::pos2;
use rack::{
    container::sizing::ModuleSize,
    devices::description::DeviceKind,
    error_window,
    widget_description::ModuleDescription,
};

use self::{
    adder::WidgetAdder,
    editor::WidgetEditor,
};

mod adder;
mod editor;

pub struct ModuleDesigner {
    module: ModuleDescription,
    widget_adder: Option<WidgetAdder>,
    editors: BTreeMap<u16, WidgetEditor>,
    saver: FileDialog,
    opener: FileDialog,
    next_wid: u16,
    error: Option<Box<dyn std::error::Error>>,
}

impl ModuleDesigner {
    pub fn new() -> Self {
        Self {
            module: ModuleDescription {
                size: ModuleSize::U1,
                visuals: Default::default(),
                devices: Default::default(),
                connections: Default::default(),
            },
            widget_adder: None,
            editors: Default::default(),
            saver: FileDialog::save_file(None),
            opener: FileDialog::open_file(None),
            next_wid: 0,
            error: None,
        }
    }

    fn save_widgets(
        &self,
        offset: Pos2,
        path: impl AsRef<Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut ws = self.module.clone();
        ws.visuals
            .iter_mut()
            .for_each(|w| w.pos -= offset.to_vec2());
        let s = serde_yaml::to_string(&ws)?;
        fs::write(path, s)?;
        Ok(())
    }

    fn load_widgets(
        &mut self,
        offset: Pos2,
        path: impl AsRef<Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let s = fs::read_to_string(path)?;
        let mut module: ModuleDescription = serde_yaml::from_str(&s)?;
        module
            .visuals
            .iter_mut()
            .for_each(|w| w.pos += offset.to_vec2());
        self.module = module;
        Ok(())
    }

    fn add_adder_with_pos(&mut self, pos: Pos2) {
        let wa = WidgetAdder::new(pos);
        match wa {
            Ok(wa) => {
                self.widget_adder = Some(wa);
            }
            Err(e) => self.error = Some(e),
        }
    }

    fn add_adder(&mut self, ctx: &Context) {
        let pos = ctx.pointer_interact_pos().unwrap_or(pos2(0., 0.));
        self.add_adder_with_pos(pos);
    }
}

impl eframe::App for ModuleDesigner {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if self.error.is_some() {
            error_window(&mut self.error, ctx);
            return;
        }

        TopBottomPanel::top("Toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ComboBox::from_label("Size")
                    .selected_text(self.module.size.to_string())
                    .show_ui(ui, |ui| {
                        for size in ModuleSize::all() {
                            ui.selectable_value(&mut self.module.size, size, size.to_string());
                        }
                    });

                ComboBox::from_label("Device")
                    .selected_text(self.module.devices.name())
                    .show_ui(ui, |ui| {
                        for dev in DeviceKind::all() {
                            ui.selectable_value(&mut self.module.device, dev, dev.name());
                        }
                    })
            });

            ui.horizontal(|ui| {
                if ui.button("Save").clicked()
                    && matches!(self.saver.state(), egui_file::State::Closed)
                {
                    self.saver.open();
                }

                if ui.button("Open").clicked()
                    && matches!(self.opener.state(), egui_file::State::Closed)
                {
                    self.opener.open();
                }

                if ui.button("Add widget").clicked() && self.widget_adder.is_none() {
                    self.add_adder_with_pos(pos2(100., 100.));
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            let r = Rect::from_min_size(ui.next_widget_position(), self.module.size.size());

            if let egui_file::State::Selected = self.saver.show(ctx).state() {
                let p = self.saver.path().unwrap().to_owned();
                if let Err(e) = self.save_widgets(r.min, p) {
                    self.error = Some(e);
                }
            }

            if let egui_file::State::Selected = self.opener.show(ctx).state() {
                let p = self.opener.path().unwrap().to_owned();
                if let Err(e) = self.load_widgets(r.min, p) {
                    self.error = Some(e);
                }
            }

            let mr = ui.allocate_rect(r, Sense::click());

            let p = ui.painter();
            p.debug_rect(
                r,
                Color32::from_rgb(100, 140, 80),
                self.module.size.to_string(),
            );

            for (id, mut w) in std::mem::take(&mut self.module.visuals)
                .into_iter()
                .enumerate()
            {
                let id = id as u16;
                let resp = ui.add(&w);

                if resp.clicked_by(PointerButton::Middle) {
                    let Vec2 { x: xs, y: ys } = resp.rect.size() * 0.125;

                    let wp = w.pos;
                    let Vec2 { x: xp, y: yp } = wp - r.min;
                    let x = (xp / xs).round() * xs;
                    let y = (yp / ys).round() * ys;
                    w.pos = r.min + vec2(x, y);
                }

                if resp.dragged_by(PointerButton::Primary) {
                    w.pos = (w.pos + resp.drag_delta()).round();
                }

                if resp.clicked_by(PointerButton::Secondary) && !self.editors.contains_key(&id) {
                    self.editors.insert(id, WidgetEditor::new(id));
                }
                self.module.visuals.push(w);
            }

            if mr.clicked_by(PointerButton::Secondary) && self.widget_adder.is_none() {
                self.add_adder(ctx)
            }

            for (wid, mut editor) in std::mem::take(&mut self.editors) {
                let w = self.module.widgets.get_mut(&wid).unwrap();
                let (delete, closing) = editor.show(ctx, self.module.device.params(), w);

                if !closing {
                    self.editors.insert(wid, editor);
                }

                if delete {
                    self.module.widgets.remove(&wid);
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
                    let wid = self.next_wid;
                    self.next_wid += 1;
                    self.module.widgets.insert(wid, widget);
                } else if wa.closing {
                } else {
                    self.widget_adder = Some(wa);
                }
            }
        });
    }
}
