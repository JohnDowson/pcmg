use std::collections::BTreeMap;

use eframe::{
    egui::{
        Sense,
        Ui,
    },
    epaint::Rect,
};
use egui::{
    InnerResponse,
    Pos2,
};

use serde::{
    Deserialize,
    Serialize,
};
use slotmap::SecondaryMap;

use crate::{
    devices::description::DeviceKind,
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
    widgets::{
        SlotWidget,
        WidgetResponse,
    },
};

use super::sizing::ModuleSize;

pub struct Module {
    pub size: ModuleSize,
    pub contents: Vec<Box<dyn SlotWidget>>,
    pub dev_kind: DeviceKind,
    pub ins: BTreeMap<usize, InputId>,
    pub outs: BTreeMap<usize, OutputId>,
    pub in_ass: SecondaryMap<InputId, usize>,
    pub out_ass: SecondaryMap<OutputId, usize>,
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
            dev_kind: DeviceKind::Output,
            ins: Default::default(),
            outs: Default::default(),
            out_ass: Default::default(),
            in_ass: Default::default(),
            values: Default::default(),
            state: Default::default(),
        }
    }

    pub fn insert_new(
        graph: &mut Graph,
        size: ModuleSize,
        dev_desc: DeviceKind,
        contents: BTreeMap<u16, WidgetDescription>,
    ) -> ModuleId {
        graph.modules.insert_with_key(|id| {
            let mut ins: BTreeMap<_, _> = Default::default();
            let mut outs: BTreeMap<_, _> = Default::default();
            let mut in_ass: SecondaryMap<_, _> = Default::default();
            let mut out_ass: SecondaryMap<_, _> = Default::default();
            let contents = contents
                .into_values()
                .enumerate()
                .map(|(i, w)| {
                    match &w.kind {
                        WidgetKind::InPort => {
                            let iid = graph.ins.insert(id);
                            ins.insert(i, iid);
                            in_ass.insert(iid, i);
                        }
                        WidgetKind::OutPort => {
                            let oid = graph.outs.insert(id);
                            outs.insert(i, oid);
                            out_ass.insert(oid, i);
                        }
                        _ => {}
                    }
                    w.dyn_widget()
                })
                .collect();
            Self {
                size,
                contents,
                dev_kind: dev_desc,
                ins,
                outs,
                in_ass,
                out_ass,
                values: vec![0.0; dev_desc.num_values()],
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

    fn ui_for(&mut self, position: Pos2, ui: &mut Ui) -> ModuleResponse {
        let mut module_res = ModuleResponse::None;
        let mut contents = std::mem::take(&mut self.contents);
        for (i, w) in contents.iter_mut().enumerate() {
            let pos = w.pos() + position.to_vec2();
            self.state.entry(i).or_default();
            ui.put(Rect::from_min_size(pos, w.size()), |ui: &mut Ui| {
                let InnerResponse { inner, response } =
                    w.show(ui, &mut self.values[w.value()], &mut self.state);

                match inner {
                    WidgetResponse::None => {}
                    WidgetResponse::Changed => {
                        module_res = ModuleResponse::Changed(i as u16, self.values[w.value()]);
                    }
                    WidgetResponse::AttemptConnectionIn => {
                        let id = self.ins.get(&i).unwrap();
                        module_res = ModuleResponse::AttemptConnectionIn((*id, i as u16));
                    }
                    WidgetResponse::AttemptConnectionOut => {
                        let id = self.outs.get(&i).unwrap();
                        module_res = ModuleResponse::AttemptConnectionOut((*id, i as u16));
                    }
                }

                response
            });
        }
        self.contents = contents;
        module_res
    }

    pub fn show(&mut self, ui: &mut Ui) -> InnerResponse<ModuleResponse> {
        let response = ui.allocate_response(self.size.size(), Sense::click_and_drag());

        let inner = self.ui_for(response.rect.min, ui);

        InnerResponse::new(inner, response)
    }
}

pub enum ModuleResponse {
    None,
    Changed(u16, f32),
    AttemptConnectionIn((InputId, u16)),
    AttemptConnectionOut((OutputId, u16)),
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
