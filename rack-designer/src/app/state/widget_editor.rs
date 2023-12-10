use egui::{
    epaint::{
        CircleShape,
        PathShape,
        RectShape,
        TextShape,
    },
    CentralPanel,
    Color32,
    Context,
    FontId,
    Rounding,
    ScrollArea,
    Sense,
    Shape,
    SidePanel,
    Stroke,
    TopBottomPanel,
};
use emath::{
    pos2,
    vec2,
    Align2,
    Pos2,
    Rect,
};
use futures::channel::mpsc::Receiver;
use rack::{
    two_drag_value,
    visuals::{
        templates::{
            VisualComponentTemplate,
            VisualShapeTemplate,
            WidgetTemplate,
        },
        Activity,
        Mode,
        VisualColor,
        VisualTheme,
    },
    widget_description::WidgetKind,
};
#[cfg(target_arch = "wasm32")]
use rack_loaders::saveloaders::save_to_url;
use rack_loaders::saveloaders::{
    loader,
    saver,
};

use crate::app::labelled_drag_value;

use super::DesignerState;

pub struct WidgetEditorState {
    state: InnerState,
    loading_chan: Option<Receiver<Option<WidgetTemplate>>>,
}

impl WidgetEditorState {
    pub fn new() -> Self {
        Self {
            state: InnerState::Edit(EditState {
                widget: WidgetTemplate::default(),
                gridsize: 10.0,
                selected_component: None,
            }),
            loading_chan: None,
        }
    }

    pub fn with_widget(widget: WidgetTemplate) -> WidgetEditorState {
        Self {
            state: InnerState::Edit(EditState {
                widget,
                gridsize: 10.0,
                selected_component: None,
            }),
            loading_chan: None,
        }
    }
}

#[derive(Default)]
enum InnerState {
    #[default]
    Empty,
    Edit(EditState),
    Save(EditState),
    Load(EditState),
    Loading(EditState),
    Preview(EditState),
    Exiting,
}

#[derive(Clone)]
struct EditState {
    widget: WidgetTemplate,
    gridsize: f32,
    selected_component: Option<usize>,
}

impl WidgetEditorState {
    pub(crate) fn show(mut self, ctx: &Context) -> DesignerState {
        self.state = match std::mem::take(&mut self.state) {
            InnerState::Empty => show_widget_empty(ctx),
            InnerState::Edit(state) => show_widget_edit(ctx, state),
            InnerState::Save(state) => {
                saver(state.widget.clone());
                InnerState::Edit(state)
            }
            InnerState::Load(state) => {
                let rx = loader();
                self.loading_chan = Some(rx);
                InnerState::Loading(state)
            }
            InnerState::Loading(state) => {
                match self.loading_chan.as_mut().map(|rx| rx.try_next()) {
                    Some(Ok(Some(Some(widget)))) => InnerState::Edit(EditState {
                        widget,
                        gridsize: state.gridsize,
                        selected_component: None,
                    }),
                    Some(Ok(Some(None))) => InnerState::Edit(state),
                    Some(Err(_)) => InnerState::Loading(state),
                    Some(Ok(None)) => panic!("Closed"),
                    None => panic!("None"),
                }
            }
            InnerState::Preview(state) => show_widget_preview(ctx, state),
            InnerState::Exiting => return DesignerState::Empty,
        };

        DesignerState::WidgetEditor(self)
    }
}

fn show_widget_preview(ctx: &Context, state: EditState) -> InnerState {
    let next = TopBottomPanel::top("toolbar")
        .show(ctx, |ui| {
            if ui.button("Back").clicked() {
                Some(InnerState::Edit(state.clone()))
            } else {
                None
            }
        })
        .inner;
    if let Some(next) = next {
        return next;
    }

    CentralPanel::default().show(ctx, |ui| {
        let center = ui.available_rect_before_wrap().center();
        state
            .widget
            .preview(ui, center, VisualTheme::default(), 0.0)
    });

    InnerState::Preview(state)
}

fn show_widget_empty(_ctx: &Context) -> InnerState {
    InnerState::Empty
}

fn show_widget_edit(ctx: &Context, mut state: EditState) -> InnerState {
    let next = TopBottomPanel::top("toolbar-widget")
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let back = ui.button("Exit to menu").clicked();
                let save = ui.button("Save").clicked();
                #[cfg(target_arch = "wasm32")]
                {
                    let export = ui.button("Get share URL").clicked();
                    if export {
                        save_to_url("pcmg_widget", state.widget.clone());
                    }
                }
                let load = ui.button("Load").clicked();
                let preview = ui.button("Preview").clicked();
                labelled_drag_value(ui, "Grid size", &mut state.gridsize);
                state.gridsize = state.gridsize.clamp(1.0, 64.0);
                if back {
                    Some(InnerState::Exiting)
                } else if save {
                    Some(InnerState::Save(state.clone()))
                } else if load {
                    Some(InnerState::Load(state.clone()))
                } else if preview {
                    Some(InnerState::Preview(state.clone()))
                } else {
                    None
                }
            })
            .inner
        })
        .inner;
    if let Some(next) = next {
        return next;
    }

    SidePanel::left("sidebar-widget")
        .resizable(false)
        .show(ctx, |ui| {
            ui.text_edit_singleline(&mut state.widget.name);

            ui.menu_button(state.widget.kind.to_string(), |ui| {
                for kind in WidgetKind::all() {
                    ui.selectable_value(&mut state.widget.kind, kind, kind.to_string());
                }
            });

            two_drag_value(
                ui,
                "Size",
                "X",
                "Y",
                &mut state.widget.size.x,
                &mut state.widget.size.y,
            );

            ui.label("Components");
            if ui.button("Add").clicked() {
                let k = state
                    .widget
                    .components
                    .last_key_value()
                    .map(|(k, _)| k + 1)
                    .unwrap_or_default();
                state
                    .widget
                    .components
                    .insert(k, VisualComponentTemplate::default());
            };
            ui.separator();
            for (i, c) in &mut state.widget.components {
                ui.selectable_value(&mut state.selected_component, Some(*i), i.to_string());

                ui.menu_button(format!("Shape: {}", &c.shape), |ui| {
                    let circle = if let shape @ VisualShapeTemplate::Circle(..) = &mut c.shape {
                        shape.clone()
                    } else {
                        VisualShapeTemplate::Circle(None, None)
                    };
                    let text = circle.to_string();
                    ui.selectable_value(&mut c.shape, circle, text);

                    let rect = if let shape @ VisualShapeTemplate::Rect(..) = &mut c.shape {
                        shape.clone()
                    } else {
                        VisualShapeTemplate::Rect(None, None, None)
                    };
                    let text = rect.to_string();
                    ui.selectable_value(&mut c.shape, rect, text);

                    let line = if let shape @ VisualShapeTemplate::Line(..) = &mut c.shape {
                        shape.clone()
                    } else {
                        VisualShapeTemplate::Line(Vec::new())
                    };
                    let text = line.to_string();
                    ui.selectable_value(&mut c.shape, line, text);

                    let text_shape = if let shape @ VisualShapeTemplate::Text(..) = &mut c.shape {
                        shape.clone()
                    } else {
                        VisualShapeTemplate::Text(None, String::new(), Default::default())
                    };
                    let text = text_shape.to_string();
                    ui.selectable_value(&mut c.shape, text_shape, text);
                });

                if let VisualShapeTemplate::Text(_, text, font) = &mut c.shape {
                    ui.text_edit_singleline(text);

                    let fonts = ui.fonts(|r| r.families());
                    ui.menu_button("Font", |ui| {
                        for new_font in fonts {
                            ui.selectable_value(font, new_font.clone(), new_font.to_string());
                        }
                    });
                }

                ui.menu_button(format!("Color: {}", &c.color), |ui| {
                    for color in VisualColor::all() {
                        ui.selectable_value(&mut c.color, color, color.to_string());
                    }
                });

                if let VisualShapeTemplate::Rect(_, _, fill) = &mut c.shape {
                    ui.menu_button("Fill color", |ui| {
                        ui.selectable_value(fill, None, "None");
                        for color in VisualColor::all() {
                            ui.selectable_value(fill, Some(color), color.to_string());
                        }
                    });
                }

                ui.menu_button(format!("Show: {}", &c.show), |ui| {
                    for show in Activity::all() {
                        ui.selectable_value(&mut c.show, show, show.to_string());
                    }
                });

                ui.menu_button(format!("Mode: {}", &c.mode), |ui| {
                    for mode in Mode::all() {
                        ui.selectable_value(&mut c.mode, mode, mode.to_string());
                    }
                });

                labelled_drag_value(ui, "Thickness", &mut c.thickness);
                ui.separator();
            }
        });

    CentralPanel::default().show(ctx, |ui| {
        ScrollArea::both().show(ui, |ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(1000., 1000.), Sense::hover());
            let center = rect.center() / state.gridsize;
            let center = center.round();
            let center = center * state.gridsize;
            ui.painter().debug_rect(
                Rect::from_center_size(center, vec2(2., 2.)),
                Color32::GOLD,
                "",
            );

            ui.painter().debug_rect(
                Rect::from_center_size(center, state.widget.size),
                Color32::BROWN,
                "",
            );

            let x = rect.min.x / state.gridsize;
            let x = x.round();
            let mut x = x * state.gridsize;
            while x < rect.max.x {
                ui.painter().line_segment(
                    [pos2(x, rect.min.y), pos2(x, rect.max.y)],
                    Stroke {
                        width: 0.1,
                        color: Color32::DARK_GREEN,
                    },
                );
                x += state.gridsize;
            }
            let mut y = rect.min.y;
            while y < rect.max.y {
                ui.painter().line_segment(
                    [pos2(rect.min.x, y), pos2(rect.max.x, y)],
                    Stroke {
                        width: 0.1,
                        color: Color32::DARK_GREEN,
                    },
                );
                y += state.gridsize;
            }

            if let Some(pos) = ui.ctx().pointer_latest_pos().map(|p| p.round()) {
                ui.painter().text(
                    pos,
                    Align2::LEFT_BOTTOM,
                    format!("{:?}{:?}", pos.round(), (pos - center.to_vec2()).round()),
                    FontId::default(),
                    Color32::WHITE,
                );
            }
            if let Some(c) = state.selected_component {
                let c = state.widget.components.get_mut(&c).unwrap();
                let (modifiers, primary) = ui.input(|r| (r.modifiers, r.pointer.primary_clicked()));

                if let Some(pointer_pos) = ctx.pointer_interact_pos() {
                    let mut pos = pointer_pos.to_vec2();
                    pos /= state.gridsize / 2.;
                    pos = pos.round();
                    pos *= state.gridsize / 2.;

                    if primary && modifiers.shift {
                        c.shape.push(pos.to_pos2() - center.to_vec2());
                    } else if primary && modifiers.ctrl {
                        c.shape.pop();
                    }
                }
            }

            for (&ci, c) in &mut state.widget.components {
                let active = state.selected_component.map(|sc| sc == ci).unwrap_or(false);
                let color = if active {
                    Color32::RED
                } else {
                    Color32::DARK_RED
                };
                let shape: Shape = match &mut c.shape {
                    VisualShapeTemplate::Line(shape) => {
                        if active {
                            for point in shape.iter_mut() {
                                let resp = ui.allocate_rect(
                                    Rect::from_center_size(
                                        center + point.to_vec2(),
                                        vec2(c.thickness, c.thickness),
                                    ),
                                    Sense::drag(),
                                );
                                ui.painter().debug_rect(resp.rect, Color32::GREEN, "");
                                *point += resp.drag_delta().round();
                            }
                        }
                        let mut shape: Vec<_> = shape
                            .iter()
                            .copied()
                            .map(|pos| center + pos.to_vec2())
                            .collect();
                        if shape.first() == shape.last() {
                            shape.pop();
                            PathShape::closed_line(
                                shape,
                                Stroke {
                                    width: c.thickness,
                                    color,
                                },
                            )
                            .into()
                        } else {
                            PathShape::line(
                                shape,
                                Stroke {
                                    width: c.thickness,
                                    color,
                                },
                            )
                            .into()
                        }
                    }
                    VisualShapeTemplate::Circle(pos, r) => {
                        if let Some(pos) = pos {
                            if active {
                                let resp = ui.allocate_rect(
                                    Rect::from_center_size(
                                        center + pos.to_vec2(),
                                        vec2(c.thickness, c.thickness),
                                    ),
                                    Sense::drag(),
                                );
                                ui.painter().debug_rect(resp.rect, Color32::GREEN, "");
                                *pos += resp.drag_delta().round();
                            }
                            if let Some(r) = r {
                                CircleShape::stroke(
                                    center + pos.to_vec2(),
                                    *r,
                                    Stroke {
                                        width: c.thickness,
                                        color,
                                    },
                                )
                                .into()
                            } else {
                                CircleShape::stroke(
                                    center + pos.to_vec2(),
                                    0.0,
                                    Stroke {
                                        width: c.thickness,
                                        color,
                                    },
                                )
                                .into()
                            }
                        } else {
                            CircleShape::stroke(Pos2::default(), 0.0, Stroke { width: 0.0, color })
                                .into()
                        }
                    }
                    VisualShapeTemplate::Text(pos, text, font) => {
                        let galley = ui.fonts(|r| {
                            r.layout_no_wrap(
                                text.clone(),
                                FontId {
                                    size: c.thickness,
                                    family: font.clone(),
                                },
                                color,
                            )
                        });
                        if let Some(pos) = pos {
                            if active {
                                let resp = ui.allocate_rect(
                                    Rect::from_center_size(center + pos.to_vec2(), galley.size()),
                                    Sense::drag(),
                                );
                                ui.painter().debug_rect(resp.rect, Color32::GREEN, "");
                                *pos += resp.drag_delta().round();
                            }
                        }

                        TextShape::new(pos.unwrap_or_default() - galley.size() / 2.0, galley).into()
                    }
                    VisualShapeTemplate::Rect(min, max, _) => {
                        if let (Some(min), Some(max)) = (min, max) {
                            if active {
                                let resp = ui.allocate_rect(
                                    Rect::from_center_size(center + min.to_vec2(), vec2(2.0, 2.0)),
                                    Sense::drag(),
                                );
                                ui.painter().debug_rect(resp.rect, Color32::GREEN, "");
                                *min += resp.drag_delta().round();

                                let resp = ui.allocate_rect(
                                    Rect::from_center_size(center + max.to_vec2(), vec2(2.0, 2.0)),
                                    Sense::drag(),
                                );
                                ui.painter().debug_rect(resp.rect, Color32::GREEN, "");
                                *max += resp.drag_delta().round();
                            }

                            RectShape::new(
                                Rect::from_min_max(center + min.to_vec2(), center + max.to_vec2()),
                                Rounding::ZERO,
                                Color32::TRANSPARENT,
                                Stroke {
                                    width: c.thickness,
                                    color,
                                },
                            )
                            .into()
                        } else {
                            Shape::Noop
                        }
                    }
                };

                ui.painter().add(shape);
            }
        });
    });
    InnerState::Edit(state)
}
