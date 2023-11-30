use std::collections::BTreeMap;

use eframe::{
    egui::{
        Response,
        Sense,
        Ui,
    },
    epaint::{
        Color32,
        Rect,
    },
};
use egui::Pos2;

use serde::{
    Deserialize,
    Serialize,
};
use slotmap::SecondaryMap;

use crate::{
    devices::{
        DeviceDescription,
        DEVICES,
    },
    graph::{
        Graph,
        InputId,
        ModuleId,
        OutputId,
    },
    widget_description::{
        ModuleDescription,
        WidgetDescription,
        WidgetKind,
    },
    widgets::SlotWidget,
};

use super::sizing::ModuleSize;

pub struct Module {
    pub size: ModuleSize,
    pub contents: Vec<Box<dyn SlotWidget>>,
    pub dev_desc: DeviceDescription,
    pub ins: SecondaryMap<InputId, u16>,
    pub outs: SecondaryMap<OutputId, u16>,
    pub values: Vec<f32>,
    pub state: SlotState,
}

pub type SlotState = BTreeMap<usize, SlotWidgetState>;
pub type SlotWidgetState = BTreeMap<&'static str, StateValue>;

impl Module {
    pub fn empty(size: ModuleSize) -> Self {
        Self {
            size,
            contents: Default::default(),
            dev_desc: DEVICES[0],
            ins: Default::default(),
            outs: Default::default(),
            values: Default::default(),
            state: Default::default(),
        }
    }

    pub fn insert_new(
        graph: &mut Graph,
        size: ModuleSize,
        dev_desc: DeviceDescription,
        contents: BTreeMap<u16, WidgetDescription>,
    ) -> ModuleId {
        graph.modules.insert_with_key(|id| {
            let mut ins: SecondaryMap<_, _> = Default::default();
            let mut outs: SecondaryMap<_, _> = Default::default();
            let contents = contents
                .into_values()
                .enumerate()
                .map(|(i, w)| {
                    match &w.kind {
                        WidgetKind::InPort => {
                            let iid = graph.ins.insert(id);
                            ins.insert(iid, i as u16);
                        }
                        WidgetKind::OutPort => {
                            let oid = graph.outs.insert(id);
                            outs.insert(oid, i as u16);
                        }
                        _ => {}
                    }
                    w.dyn_widget()
                })
                .collect();
            Self {
                size,
                contents,
                dev_desc,
                ins,
                outs,
                values: vec![0.0; dev_desc.params.len()],
                state: Default::default(),
            }
        })
    }

    pub fn insert_from_description(graph: &mut Graph, description: ModuleDescription) -> ModuleId {
        let ModuleDescription {
            size,
            device,
            widgets,
        } = description;
        Self::insert_new(graph, size, device, widgets)
    }

    fn ui_for(&mut self, position: Pos2, ui: &mut Ui) {
        let mut contents = std::mem::take(&mut self.contents);
        for (i, w) in contents.iter_mut().enumerate() {
            let pos = w.pos() + position.to_vec2();
            self.state.entry(i).or_default();
            ui.put(Rect::from_min_size(pos, w.size()), |ui: &mut Ui| {
                w.show(ui, &mut self.values[w.value()], &mut self.state)
            });
        }
        self.contents = contents;
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let size = self.size.size();

        let resp = ui.allocate_response(self.size.size(), Sense::click_and_drag());

        self.ui_for(resp.rect.min, ui);

        resp
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StateValue {
    Float(f32),
    Bool(bool),
    Range(f32, f32),
}

impl std::fmt::Display for StateValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateValue::Float(v) => v.fmt(f),
            StateValue::Bool(b) => b.fmt(f),
            StateValue::Range(s, e) => write!(f, "{s}..{e}"),
        }
    }
}

impl std::str::FromStr for StateValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s {
            "true" => Self::Bool(true),
            "false" => Self::Bool(false),
            s => {
                let res = s
                    .split("..")
                    .map(|fv| fv.parse())
                    .collect::<Result<Vec<f32>, _>>()
                    .map_err(|_| ())?;
                match res.len() {
                    1 => Self::Float(res[0]),
                    2 => Self::Range(res[0], res[1]),
                    _ => return Err(()),
                }
            }
        };
        Ok(v)
    }
}
