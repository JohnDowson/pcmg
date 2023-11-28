use std::collections::BTreeMap;

use eframe::{
    egui::{Sense, Widget},
    epaint::{Color32, Pos2, Rect},
};
use egui::{vec2, Align2, Response, Rounding, Ui, Vec2};
use serde::{Deserialize, Serialize};

use crate::{
    container::{sizing::SlotSize, StateValue},
    widgets::{
        connector::{InPort, OutPort},
        knob::Knob,
        SlotWidget,
    },
};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ModuleDescription {
    pub size: SlotSize,
    pub widgets: BTreeMap<Wid, WidgetDescription>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Wid(pub u16);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Sid(pub u16);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct WidFull {
    pub sid: Sid,
    pub wid: Wid,
}

pub fn wid_full(sid: Sid, wid: Wid) -> WidFull {
    WidFull { sid, wid }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum WidgetKind {
    Knob,
    InPort,
    OutPort,
}

impl std::fmt::Display for WidgetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl WidgetKind {
    pub fn all() -> [Self; 3] {
        use WidgetKind::*;
        [Knob, InPort, OutPort]
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WidgetVisual {
    pub kind: WidgetVisualKind,
    pub mode: WidgetVisualMode,
    pub center: Pos2,
}

impl WidgetVisual {
    pub fn show(&self, ui: &mut Ui, c: Pos2) -> Response {
        let cnd = Sense::click_and_drag();
        let center = c + self.center.to_vec2();
        let resp = match self.kind {
            WidgetVisualKind::Point => {
                let r = ui.allocate_rect(Rect::from_center_size(center, vec2(2., 2.)), cnd);
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
                    cnd,
                );
                ui.painter()
                    .circle_stroke(center, radius, ui.visuals().widgets.active.fg_stroke);
                r
            }
            WidgetVisualKind::Rect(size) => {
                let r = ui.allocate_rect(Rect::from_center_size(center, size), cnd);
                ui.painter().rect_stroke(
                    Rect::from_center_size(center, size),
                    Rounding::none(),
                    ui.visuals().widgets.active.fg_stroke,
                );
                r
            }
            WidgetVisualKind::Line(length, angle) => {
                let (mut a, mut b) = (self.center, self.center);
                let angle = angle.to_radians();
                a.x += length / 2. * angle.sin();
                a.y += length / 2. * angle.cos();
                b.x -= length / 2. * angle.sin();
                b.y -= length / 2. * angle.cos();
                let (a, b) = (c + a.to_vec2(), c + b.to_vec2());

                let r = ui.allocate_rect(Rect::from_two_pos(a, b).expand(1.0), cnd);
                ui.painter()
                    .line_segment([a, b], ui.visuals().widgets.active.fg_stroke);
                r
            }
            WidgetVisualKind::Readout(_) => todo!(),
            WidgetVisualKind::Text(ref t) => {
                let r = ui.allocate_rect(Rect::from_center_size(center, vec2(2., 2.)), cnd);
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
                let r = ui.allocate_rect(Rect::from_center_size(center, vec2(2., 2.)), cnd);
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
    Line(f32, f32),
    Readout(f32),
    Text(String),
    Symbol(char),
}

impl std::fmt::Display for WidgetVisualKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl WidgetVisualKind {
    pub fn all() -> [Self; 7] {
        use WidgetVisualKind::*;
        [
            Point,
            Circle(Default::default()),
            Rect(Default::default()),
            Line(Default::default(), Default::default()),
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

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WidgetDescription {
    pub kind: WidgetKind,
    pub name: String,
    #[serde(default)]
    pub pos: Pos2,
    pub size: Vec2,
    #[serde(serialize_with = "crate::ser_btree_as_vec")]
    #[serde(deserialize_with = "crate::de_vec_as_btree")]
    pub visuals: BTreeMap<usize, WidgetVisual>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, StateValue>,
}

impl WidgetDescription {
    pub fn new(
        kind: WidgetKind,
        name: String,
        pos: Pos2,
        size: Vec2,
        visuals: BTreeMap<usize, WidgetVisual>,
        extra: BTreeMap<String, StateValue>,
    ) -> Self {
        Self {
            kind,
            name,
            pos,
            size,
            visuals,
            extra,
        }
    }

    pub fn dyn_widget(&self, id: WidFull) -> Box<dyn SlotWidget> {
        match self.kind {
            WidgetKind::Knob => Knob::from_description(id, self).map(Box::new).unwrap(),
            WidgetKind::InPort => InPort::from_description(id, self).map(Box::new).unwrap(),
            WidgetKind::OutPort => OutPort::from_description(id, self).map(Box::new).unwrap(),
        }
    }
}

impl Widget for &WidgetDescription {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let resp = ui.allocate_rect(
            Rect::from_min_size(self.pos, self.dyn_widget(wid_full(Sid(0), Wid(0))).size()),
            Sense::click_and_drag(),
        );

        let p = ui.painter();
        p.debug_rect(resp.rect, Color32::from_rgb(180, 170, 100), &self.name);

        resp
    }
}
