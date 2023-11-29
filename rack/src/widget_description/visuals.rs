use eframe::{
    egui::Sense,
    epaint::{
        Pos2,
        Rect,
    },
};
use egui::{
    vec2,
    Align2,
    Response,
    Rounding,
    Ui,
    Vec2,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WidgetVisual {
    pub kind: WidgetVisualKind,
    pub mode: WidgetVisualMode,
    pub center: Pos2,
}

impl WidgetVisual {
    pub fn show(&self, ui: &mut Ui, c: Pos2, sense: Sense) -> Response {
        let center = c + self.center.to_vec2();
        let resp = match self.kind {
            WidgetVisualKind::Point => {
                let r = ui.allocate_rect(Rect::from_center_size(center, vec2(2., 2.)), sense);
                ui.painter().circle_filled(
                    center,
                    1.0,
                    ui.visuals().widgets.active.fg_stroke.color,
                );
                r
            }
            WidgetVisualKind::Circle(radius) => {
                let r = ui.allocate_rect(
                    Rect::from_center_size(center, vec2(radius * 2.0, radius * 2.0)),
                    sense,
                );
                ui.painter()
                    .circle_stroke(center, radius, ui.visuals().widgets.active.fg_stroke);
                r
            }
            WidgetVisualKind::Rect(size) => {
                let r = ui.allocate_rect(Rect::from_center_size(center, size), sense);
                ui.painter().rect_stroke(
                    Rect::from_center_size(center, size),
                    Rounding::none(),
                    ui.visuals().widgets.active.fg_stroke,
                );
                r
            }
            WidgetVisualKind::Line(end) => {
                let (a, b) = (center, c + end.to_vec2());

                let r = ui.allocate_rect(Rect::from_two_pos(a, b).expand(1.0), sense);
                ui.painter()
                    .line_segment([a, b], ui.visuals().widgets.active.fg_stroke);
                r
            }
            WidgetVisualKind::Readout(size) => {
                let r = ui.allocate_rect(Rect::from_center_size(center, vec2(2., 2.)), sense);
                let font = egui::FontId {
                    size,
                    ..Default::default()
                };
                ui.painter().text(
                    center,
                    Align2::CENTER_CENTER,
                    "readout",
                    font,
                    ui.visuals().widgets.active.fg_stroke.color,
                );
                r
            }
            WidgetVisualKind::Text(ref t) => {
                let r = ui.allocate_rect(Rect::from_center_size(center, vec2(2., 2.)), sense);
                ui.painter().text(
                    center,
                    Align2::CENTER_CENTER,
                    t,
                    Default::default(),
                    ui.visuals().widgets.active.fg_stroke.color,
                );
                r
            }
            WidgetVisualKind::Symbol(c) => {
                let r = ui.allocate_rect(Rect::from_center_size(center, vec2(2., 2.)), sense);
                ui.painter().text(
                    center,
                    Align2::CENTER_CENTER,
                    c,
                    Default::default(),
                    ui.visuals().widgets.active.fg_stroke.color,
                );
                r
            }
        };
        resp
    }
}

/// Positions are relative from widget center
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum WidgetVisualKind {
    Point,
    Circle(f32),
    Rect(Vec2),
    Line(Pos2),
    Readout(f32),
    Text(String),
    Symbol(char),
}

impl std::fmt::Display for WidgetVisualKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WidgetVisualKind::Point => f.write_str("Point"),
            WidgetVisualKind::Circle(_) => f.write_str("Circle"),
            WidgetVisualKind::Rect(_) => f.write_str("Rect"),
            WidgetVisualKind::Line(_) => f.write_str("Line"),
            WidgetVisualKind::Readout(_) => f.write_str("Readout"),
            WidgetVisualKind::Text(_) => f.write_str("Text"),
            WidgetVisualKind::Symbol(_) => f.write_str("Symbol"),
        }
    }
}

impl WidgetVisualKind {
    pub fn all() -> [Self; 7] {
        use WidgetVisualKind::*;
        [
            Point,
            Circle(Default::default()),
            Rect(Default::default()),
            Line(Default::default()),
            Readout(Default::default()),
            Text(Default::default()),
            Symbol(Default::default()),
        ]
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum WidgetVisualMode {
    Static,
    StateRelative,
}

impl WidgetVisualMode {
    pub fn all() -> [Self; 2] {
        use WidgetVisualMode::*;
        [Static, StateRelative]
    }
}

impl std::fmt::Display for WidgetVisualMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
