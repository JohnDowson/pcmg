use std::{
    collections::BTreeMap,
    fs,
    path::Path,
};

use egui::{
    vec2,
    CentralPanel,
    Color32,
    ComboBox,
    Context,
    DragValue,
    PointerButton,
    Sense,
    Slider,
    TopBottomPanel,
    Ui,
    Vec2,
};
use egui_file::FileDialog;
use rack::{
    error_window,
    vec_drag_value,
    widget_description::{
        visuals::{
            WidgetVisual,
            WidgetVisualKind,
            WidgetVisualMode,
        },
        KnobKind,
        WidgetDescription,
        WidgetKind,
    },
    widget_name,
    widget_prefabs,
};
use uuid::Uuid;

use creator::WidgetCreator;

use self::creator::VisualCreator;

mod creator;

pub struct WidgetDesigner {
    widget: Option<Uuid>,
    widgets: BTreeMap<Uuid, WidgetDescription>,
    saver: FileDialog,
    opener: FileDialog,
    creator: Option<WidgetCreator>,
    editor: Option<VisualCreator>,
    error: Option<Box<dyn std::error::Error>>,
}

impl WidgetDesigner {
    pub fn new() -> Self {
        Self {
            widget: None,
            widgets: Default::default(),
            saver: FileDialog::save_file(None),
            opener: FileDialog::open_file(None),
            creator: None,
            editor: None,
            error: None,
        }
    }

    fn load(&mut self, p: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let ws = widget_prefabs(&p)?;
        self.widgets.extend(ws);
        Ok(())
    }

    fn save(&mut self, p: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let s = serde_yaml::to_string(&self.widgets)?;
        fs::write(p, s)?;
        Ok(())
    }

    fn update_with_widget(&mut self, uuid: Uuid, ctx: &Context, ui: &mut Ui) {
        let w = self.widgets.get_mut(&uuid).unwrap();

        let (r, resp) = ui.allocate_exact_size(w.size, Sense::click());
        let c = r.center();

        let p = ui.painter_at(r);

        p.debug_rect(r, Color32::from_rgb(60, 140, 60), "");

        for (id, visual) in &mut w.visuals {
            let resp = visual.show(ui, c, Sense::click_and_drag());

            visual.center = (visual.center + resp.drag_delta()).round();

            if resp.clicked_by(PointerButton::Middle) {
                let Vec2 { x: xs, y: ys } = resp.rect.size() * 0.25;

                let wp = w.pos + resp.drag_delta();
                let Vec2 { x: xp, y: yp } = wp - r.min;
                let x = (xp / xs).round() * xs;
                let y = (yp / ys).round() * ys;
                w.pos = r.min + vec2(x, y);
            }

            if resp.clicked_by(PointerButton::Secondary) && self.editor.is_none() {
                self.editor = Some(VisualCreator::new(*id, "Edit"))
            }
        }

        if resp.clicked_by(PointerButton::Secondary) && self.editor.is_none() {
            Self::add_visual(&mut self.editor, w);
        }

        if let Some(editor) = &self.editor {
            let id = editor.id;
            let (remove, closing) = editor.show(ctx, w.visuals.get_mut(&id).unwrap());

            if remove {
                w.visuals.remove(&id);
            }

            if closing {
                self.editor.take();
            }
        }
    }

    fn add_visual(editor: &mut Option<VisualCreator>, w: &mut WidgetDescription) {
        let id = w
            .visuals
            .last_key_value()
            .map(|(id, _)| *id + 1)
            .unwrap_or_default();
        w.visuals.insert(
            id,
            WidgetVisual {
                kind: WidgetVisualKind::Point,
                mode: WidgetVisualMode::Static,
                center: Default::default(),
                style: Default::default(),
            },
        );

        *editor = Some(VisualCreator::new(id, "New"))
    }
}

impl eframe::App for WidgetDesigner {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if self.error.is_some() {
            error_window(&mut self.error, ctx);
            return;
        }

        TopBottomPanel::top("Toolbar").show(ctx, |ui| {
            ComboBox::from_label("Widget")
                .width(256.0)
                .selected_text(widget_name(self.widget, &self.widgets))
                .show_ui(ui, |ui| {
                    for uuid in self.widgets.keys() {
                        ui.selectable_value(
                            &mut self.widget,
                            Some(*uuid),
                            widget_name(Some(*uuid), &self.widgets),
                        );
                    }
                });

            ui.horizontal(|ui| {
                let save = ui.add_enabled(self.widget.is_some(), |ui: &mut Ui| ui.button("Save"));
                if save.clicked() && matches!(self.saver.state(), egui_file::State::Closed) {
                    self.saver.open();
                }

                if ui.button("Open").clicked()
                    && matches!(self.opener.state(), egui_file::State::Closed)
                {
                    self.opener.open();
                }

                if ui.button("New").clicked() && self.creator.is_none() {
                    self.creator = Some(WidgetCreator::new());
                }

                if let Some(w) = &self.widget {
                    let w = self.widgets.get_mut(w).unwrap();
                    if ui.button("Add visual").clicked() && self.editor.is_none() {
                        Self::add_visual(&mut self.editor, w);
                    }
                    ui.vertical(|ui| {
                        vec_drag_value(ui, "Size", &mut w.size);
                    });
                    ui.vertical(|ui| match &mut w.kind {
                        WidgetKind::Knob(KnobKind {
                            value_range: (v_s, v_e),
                            angle_range: (a_s, a_e),
                            default_pos,
                            speed,
                        }) => {
                            ui.horizontal(|ui| {
                                ui.label("Value range");
                                ui.add(DragValue::new(v_s));
                                ui.add(DragValue::new(v_e));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Angle range");
                                ui.add(DragValue::new(a_s).clamp_range(0.0..=360.));
                                ui.add(DragValue::new(a_e).clamp_range(0.0..=360.));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Default position");
                                ui.add(Slider::new(default_pos, 0.0..=1.0));
                                ui.label("Speed");
                                ui.add(Slider::new(speed, 0.0..=1.0));
                            });
                        }
                        WidgetKind::Port => {}
                    });
                }
            })
        });

        CentralPanel::default().show(ctx, |ui| {
            if let egui_file::State::Selected = self.saver.show(ctx).state() {
                let p = self.saver.path().unwrap().to_owned();
                if let Err(e) = self.save(p) {
                    self.error = Some(e);
                };
            }

            if let egui_file::State::Selected = self.opener.show(ctx).state() {
                let p = self.opener.path().unwrap().to_owned();
                if let Err(e) = self.load(p) {
                    self.error = Some(e);
                };
            }

            if let Some(mut creator) = self.creator.take() {
                creator.show(ctx);

                if let WidgetCreator {
                    uuid,
                    closing: true,
                    kind,
                    name,
                    size,
                } = creator
                {
                    let w = WidgetDescription {
                        kind,
                        name,
                        pos: Default::default(),
                        size,
                        visuals: Default::default(),
                    };
                    self.widgets.insert(uuid, w);
                } else {
                    self.creator = Some(creator);
                }
            }

            if let Some(uuid) = &self.widget {
                self.update_with_widget(*uuid, ctx, ui);
            } else {
                ui.label("No widget loaded");
            }
        });
    }
}
