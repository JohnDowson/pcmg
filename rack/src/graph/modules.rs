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
    module_description::ModuleDescription,
    visuals::{
        templates::WidgetTemplate,
        VisualTheme,
    },
    widgets::{
        SlotWidget,
        WidgetResponse,
    },
    Tooltipable,
};

use super::{
    Connector,
    DeviceId,
    InputId,
    ModuleId,
    OutputId,
    VisualId,
};

pub struct Module {
    pub size: ModuleSize,
    pub devices: Vec<DeviceId>,
    pub visuals: SlotMap<VisualId, SlotWidget>,
    pub theme: VisualTheme,
    pub values: SecondaryMap<VisualId, Connector>,
    /// Maps inputs to their visuals
    pub ins: SecondaryMap<InputId, VisualId>,
    /// Maps outputs to their visuals
    pub outs: SecondaryMap<OutputId, VisualId>,
}

impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("size", &self.size)
            .field("devices", &self.devices)
            .field("visuals_count", &self.visuals.len())
            .field("values", &self.values)
            .field("ins", &self.ins)
            .field("outs", &self.outs)
            .finish()
    }
}

impl Module {
    fn insert_new(
        graph: &mut Graph,
        size: ModuleSize,
        visual_descs: BTreeMap<usize, WidgetTemplate>,
        theme: VisualTheme,
        devices: BTreeMap<usize, DeviceKind>,
        mut connections: BTreeMap<(usize, usize), usize>,
    ) -> ModuleId {
        let mut visuals = SlotMap::default();
        let mut visual_ids = BTreeMap::default();
        for (i, desc) in visual_descs {
            visual_ids.insert(i, visuals.insert(desc.into_slot_widget().unwrap()));
        }

        let mut values = SecondaryMap::default();
        let mut ins = SecondaryMap::default();
        let mut outs = SecondaryMap::default();
        let devices = devices
            .into_iter()
            .map(|(di, device)| {
                graph.devices.insert_with_key(|did| {
                    let params = device.params();
                    for (pi, param) in params.iter().enumerate() {
                        match param {
                            Param::In(_) => {
                                let param = graph.ins.insert((did, pi as u8));
                                graph.dev_ins.entry(did).unwrap().or_default().push(param);
                                if let Some(vi) = connections.remove(&(di, pi)) {
                                    let vid = visual_ids[&vi];
                                    values.insert(vid, Connector::In(param));
                                    ins.insert(param, vid);
                                }
                            }
                            Param::Out(_) => {
                                let param = graph.outs.insert((did, pi as u8));
                                graph.dev_outs.entry(did).unwrap().or_default().push(param);
                                if let Some(vi) = connections.remove(&(di, pi)) {
                                    let vid = visual_ids[&vi];
                                    values.insert(vid, Connector::Out(param));
                                    outs.insert(param, vid);
                                }
                            }
                        }
                    }

                    device
                })
            })
            .collect();

        graph.modules.insert(Self {
            size,
            devices,
            visuals,
            theme,
            values,
            ins,
            outs,
        })
    }

    pub fn insert_from_description(graph: &mut Graph, description: ModuleDescription) -> ModuleId {
        let ModuleDescription {
            uuid: _,
            name: _,
            theme,
            size,
            visuals,
            devices,
            connections,
        } = description;
        Self::insert_new(graph, size, visuals, theme, devices, connections)
    }

    fn ui_for(&mut self, position: Pos2, ui: &mut Ui) -> ModuleResponse {
        let mut module_res = ModuleResponse::None;
        let mut visuals = std::mem::take(&mut self.visuals);
        for (vid, w) in visuals.iter_mut() {
            let pos = w.pos() + position.to_vec2();
            ui.put(Rect::from_min_size(pos, w.size()), |ui: &mut Ui| {
                let InnerResponse { inner, response } = w.show(ui, self.theme);
                if let Some(connected_plug) = self.values.get(vid).copied() {
                    match inner {
                        WidgetResponse::None => {}
                        WidgetResponse::Changed => {
                            module_res = ModuleResponse::Changed(connected_plug, w.value());
                        }
                        WidgetResponse::AttemptConnection => {
                            module_res = ModuleResponse::AttemptConnection(connected_plug);
                        }
                        WidgetResponse::AttemptDisconnect => {
                            module_res = ModuleResponse::AttemptDisconnect(connected_plug)
                        }
                    }
                }
                response.on_hover_text(w.tooltip())
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
    AttemptDisconnect(Connector),
}
