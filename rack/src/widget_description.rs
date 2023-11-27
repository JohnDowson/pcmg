use std::collections::BTreeMap;

use eframe::{
    egui::{Sense, Widget},
    epaint::{Color32, Pos2, Rect},
};
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
    pub widgets: Vec<WidgetDescription>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Wid(pub u16);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Sid(pub u16);

fn zero_wid() -> Wid {
    Wid(0)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct WidFull {
    pub sid: Sid,
    pub wid: Wid,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum WidgetKind {
    Knob,
    InPort,
    OutPort,
}

impl WidgetKind {
    pub fn all() -> [Self; 1] {
        use WidgetKind::*;
        [Knob]
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WidgetDescription {
    pub kind: WidgetKind,
    #[serde(default = "zero_wid")]
    pub wid: Wid,
    pub name: String,
    #[serde(default)]
    pub pos: Pos2,
    #[serde(flatten)]
    pub extra: BTreeMap<String, StateValue>,
}

impl WidgetDescription {
    pub fn new(
        kind: WidgetKind,
        wid: Wid,
        name: String,
        pos: Pos2,
        extra: BTreeMap<String, StateValue>,
    ) -> Self {
        Self {
            kind,
            wid,
            name,
            pos,
            extra,
        }
    }

    pub fn dyn_widget(&self, sid: Sid) -> Box<dyn SlotWidget> {
        match self.kind {
            WidgetKind::Knob => Knob::from_description(sid, self).map(Box::new).unwrap(),
            WidgetKind::InPort => InPort::from_description(sid, self).map(Box::new).unwrap(),
            WidgetKind::OutPort => OutPort::from_description(sid, self).map(Box::new).unwrap(),
        }
    }
}

impl Widget for &mut WidgetDescription {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let resp = ui.allocate_rect(
            Rect::from_min_size(self.pos, self.dyn_widget(Sid(0)).size()),
            Sense::click_and_drag(),
        );
        let d = resp.drag_delta();
        self.pos += d;
        self.pos = self.pos.round();

        let p = ui.painter();
        p.debug_rect(resp.rect, Color32::from_rgb(180, 170, 100), &self.name);

        resp
    }
}
