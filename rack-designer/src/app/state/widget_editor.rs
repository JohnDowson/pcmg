use egui::{
    epaint::{
        CircleShape,
        PathShape,
    },
    CentralPanel,
    Color32,
    Context,
    FontId,
    Sense,
    Shape,
    SidePanel,
    Stroke,
    TopBottomPanel,
};
use emath::{
    vec2,
    Align2,
    Pos2,
    Rect,
};
use rack::visuals::{
    Activity,
    VisualColor,
    VisualComponent,
    VisualShape,
    WidgetTemplate,
};

use crate::app::labelled_drag_value;

use super::DesignerState;

pub struct WidgetEditorState {
    state: InnerState,
}

impl WidgetEditorState {
    pub fn new() -> Self {
        Self {
            state: InnerState::Edit(EditState {
                widget: WidgetTemplate::default(),
                selected_component: None,
            }),
        }
    }
}

#[derive(Default)]
enum InnerState {
    #[default]
    Empty,
    Edit(EditState),
}

struct EditState {
    widget: WidgetTemplate,
    selected_component: Option<usize>,
}

impl WidgetEditorState {
    pub(crate) fn show(mut self, ctx: &Context) -> DesignerState {
        self.state = match std::mem::take(&mut self.state) {
            InnerState::Empty => show_widget_empty(ctx),
            InnerState::Edit(state) => show_widget_edit(ctx, state),
        };

        DesignerState::WidgetEditor(self)
    }
}

fn show_widget_empty(ctx: &Context) -> InnerState {
    CentralPanel::default().show(ctx, |_ui| {});

    InnerState::Empty
}

fn show_widget_edit(ctx: &Context, mut state: EditState) -> InnerState {
    TopBottomPanel::top("toolbar-widget").show(ctx, |_ui| {});

    SidePanel::left("sidebar-widget")
        .resizable(false)
        .show(ctx, |ui| {
            //
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
                    .insert(k, VisualComponent::default());
            };
            ui.separator();
            for (i, c) in &mut state.widget.components {
                ui.selectable_value(&mut state.selected_component, Some(*i), i.to_string());

                ui.menu_button(format!("Shape: {}", &c.shape), |ui| {
                    let circle = if let shape @ VisualShape::Circle(..) = &mut c.shape {
                        shape.clone()
                    } else {
                        VisualShape::Circle(None, None)
                    };
                    let text = circle.to_string();
                    ui.selectable_value(&mut c.shape, circle, text);
                    let line = if let shape @ VisualShape::Line(..) = &mut c.shape {
                        shape.clone()
                    } else {
                        VisualShape::Line(Vec::new())
                    };
                    let text = line.to_string();
                    ui.selectable_value(&mut c.shape, line, text);
                });

                ui.menu_button(format!("Color: {}", &c.color), |ui| {
                    for color in VisualColor::all() {
                        ui.selectable_value(&mut c.color, color, color.to_string());
                    }
                });

                ui.menu_button(format!("Show: {}", &c.show), |ui| {
                    for show in Activity::all() {
                        ui.selectable_value(&mut c.show, show, show.to_string());
                    }
                });
                labelled_drag_value(ui, "Thickness", &mut c.thickness);
                ui.separator();
            }
        });

    CentralPanel::default().show(ctx, |ui| {
        let rect = ui.available_rect_before_wrap();
        let center = rect.center().round();
        if let Some(pos) = ui.ctx().pointer_latest_pos().map(|p| p.round()) {
            ui.painter().text(
                pos,
                Align2::LEFT_BOTTOM,
                format!("{:?}{:?}", pos.round(), pos.round() - center.to_vec2()),
                FontId::default(),
                Color32::WHITE,
            );
        }
        if let Some(c) = state.selected_component {
            let c = state.widget.components.get_mut(&c).unwrap();
            let (modifiers, dragging, primary) = ui.input(|r| {
                (
                    r.modifiers,
                    r.pointer.is_decidedly_dragging(),
                    r.pointer.primary_clicked(),
                )
            });

            if let Some(pointer_pos) = ctx.pointer_interact_pos() {
                if primary && modifiers.shift {
                    c.shape
                        .push(center + (pointer_pos.round().to_vec2() - center.to_vec2()));
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
                VisualShape::Line(shape) => {
                    if active {
                        for point in shape.iter_mut() {
                            let resp = ui.allocate_rect(
                                Rect::from_center_size(*point, vec2(c.thickness, c.thickness)),
                                Sense::drag(),
                            );
                            ui.painter().debug_rect(resp.rect, Color32::GREEN, "");
                            *point += resp.drag_delta().round();
                        }
                    }
                    let mut shape: Vec<_> = shape
                        .iter()
                        .copied()
                        .map(|pos| center + (pos.to_vec2() - center.to_vec2()))
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
                VisualShape::Circle(pos, r) => {
                    if let Some(pos) = pos {
                        if active {
                            let resp = ui.allocate_rect(
                                Rect::from_center_size(*pos, vec2(c.thickness, c.thickness)),
                                Sense::drag(),
                            );
                            ui.painter().debug_rect(resp.rect, Color32::GREEN, "");
                            *pos += resp.drag_delta().round();
                        }
                        if let Some(r) = r {
                            CircleShape::stroke(
                                *pos,
                                *r,
                                Stroke {
                                    width: c.thickness,
                                    color,
                                },
                            )
                            .into()
                        } else {
                            CircleShape::stroke(
                                *pos,
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
            };

            ui.painter().add(shape);
        }
    });

    InnerState::Edit(state)
}
