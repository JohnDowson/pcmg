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
use slotmap::{
    SecondaryMap,
    SlotMap,
};

use crate::{
    container::sizing::ModuleSize,
    devices::description::{
        DeviceKind,
        Param,
    },
    graph::Graph,
    widget_description::{
        ModuleDescription,
        WidgetDescription,
    },
    widgets::{
        SlotWidget,
        WidgetResponse,
    },
};

use super::{
    nodes::Node,
    Connector,
    InputId,
    ModuleId,
    NodeId,
    OutputId,
    VisualId,
};

pub struct Module {
    pub size: ModuleSize,
    pub node: NodeId,
    pub visuals: SlotMap<VisualId, Box<dyn SlotWidget>>,
    pub values: SecondaryMap<VisualId, Connector>,
    /// Maps inputs to their visuals
    pub ins: SecondaryMap<InputId, VisualId>,
    /// Maps outputs to their visuals
    pub outs: SecondaryMap<OutputId, VisualId>,
}

impl Module {
    fn insert_new(
        graph: &mut Graph,
        size: ModuleSize,
        mut visual_descs: Vec<WidgetDescription>,
        devices: Vec<DeviceKind>,
        mut connections: BTreeMap<(usize, usize), usize>,
    ) -> ModuleId {
        let mut visuals = SlotMap::default();
        let mut values = SecondaryMap::default();
        let mut ins = SecondaryMap::default();
        let mut outs = SecondaryMap::default();

        let mut node = Node::empty();
        let node = graph.nodes.insert_with_key(|nid| {
            for (di, device) in devices.into_iter().enumerate() {
                let params = device.params();
                let did = node.devices.insert(device);
                for (pi, param) in params.iter().enumerate() {
                    match param {
                        Param::In(_) => {
                            let param = graph.ins.insert(nid);
                            node.input_to_param.insert(param, (did, pi));
                            if let Some(vi) = connections.remove(&(di, pi)) {
                                let vid = visuals.insert(visual_descs.remove(vi).dyn_widget());
                                values.insert(vid, Connector::In(param));
                                ins.insert(param, vid);
                            }
                        }
                        Param::Out(_) => {
                            let param = graph.outs.insert(nid);
                            node.output_to_param.insert(param, (did, pi));
                            if let Some(vi) = connections.remove(&(di, pi)) {
                                let vid = visuals.insert(visual_descs.remove(vi).dyn_widget());
                                values.insert(vid, Connector::Out(param));
                                outs.insert(param, vid);
                            }
                        }
                    }
                }
            }
            node
        });

        graph.modules.insert(Self {
            size,
            node,
            visuals,
            values,
            ins,
            outs,
        })
    }

    pub fn insert_from_description(graph: &mut Graph, description: ModuleDescription) -> ModuleId {
        let ModuleDescription {
            size,
            visuals,
            devices,
            connections,
        } = description;
        Self::insert_new(graph, size, visuals, devices, connections)
    }

    fn ui_for(&mut self, position: Pos2, ui: &mut Ui) -> ModuleResponse {
        let mut module_res = ModuleResponse::None;
        let mut visuals = std::mem::take(&mut self.visuals);
        for (vid, w) in visuals.iter_mut() {
            let pos = w.pos() + position.to_vec2();
            ui.put(Rect::from_min_size(pos, w.size()), |ui: &mut Ui| {
                let InnerResponse { inner, response } = w.show(ui);
                if let Some(connected_plug) = self.values.get(vid).copied() {
                    match inner {
                        WidgetResponse::None => {}
                        WidgetResponse::Changed => {
                            module_res = ModuleResponse::Changed(connected_plug, w.value());
                        }
                        WidgetResponse::AttemptConnection => {
                            module_res = ModuleResponse::AttemptConnection(connected_plug);
                        }
                    }
                }

                response
            });
        }
        self.visuals = visuals;
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
    Changed(Connector, f32),
    AttemptConnection(Connector),
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