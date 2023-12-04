use egui::{
    epaint::PathShape,
    CentralPanel,
    Color32,
    Context,
    FontId,
    Shape,
    SidePanel,
    Stroke,
    TopBottomPanel,
};
use emath::Align2;
use rack::visuals::{
    Activity,
    VisualColor,
    VisualComponent,
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
        if let Some(pos) = ui.ctx().pointer_latest_pos().map(|p| p.round()) {
            ui.painter().text(
                pos,
                Align2::LEFT_BOTTOM,
                format!("{pos:?}"),
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
                    c.shape.push(pointer_pos.round());
                } else if primary && modifiers.ctrl {
                    c.shape.pop();
                }
            }
        }

        for c in state.widget.components.values() {
            let shape = if c.shape.first() == c.shape.last() {
                let mut shape = c.shape.clone();
                shape.pop();
                PathShape::closed_line(
                    shape,
                    Stroke {
                        width: c.thickness,
                        color: Color32::RED,
                    },
                )
            } else {
                PathShape::line(
                    c.shape.clone(),
                    Stroke {
                        width: c.thickness,
                        color: Color32::RED,
                    },
                )
            };
            ui.painter().add(shape);
        }
    });

    InnerState::Edit(state)
}
