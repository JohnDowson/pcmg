use crate::devices::{
    description::DeviceKind,
    impls::{
        Control,
        Output,
    },
    Device,
    DEVICES,
};
use fusebox::FuseBox;
use std::collections::BTreeMap;

use super::CtlGraph;

type NodeToDevice = BTreeMap<u16, usize>;
type OutputMap = BTreeMap<u16, Vec<(u16, u8)>>;

pub struct ByteCode {
    devices: FuseBox<dyn Device + Send + Sync>,
    node_to_device: NodeToDevice,
    code: Vec<Op>,
    sample: f32,
}

impl std::fmt::Debug for ByteCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ByteCode")
            .field("devices_count", &self.devices.len())
            .field("node_to_device", &self.node_to_device)
            .field("code", &self.code)
            .field("sample", &self.sample)
            .finish()
    }
}

impl ByteCode {
    pub fn update_param(&mut self, (dev, param): (u16, u16), value: f32) {
        let d = self.node_to_device[&dev];
        if let Some(d) = self.devices.get_mut(d) {
            d.set_param_indexed(param as u8, value)
        }
    }

    pub fn sample(&mut self) -> f32 {
        for op in &self.code {
            match op {
                Op::Sample(d) => {
                    let did = self.node_to_device[d];
                    self.sample = self.devices[did].output();
                }
                Op::Output => break,
                Op::Parametrise(d, pid) => {
                    self.devices[*d as usize].set_param_indexed(*pid, self.sample)
                }
            }
        }
        self.sample
    }
}

#[derive(Debug)]
enum Op {
    Sample(u16),
    Output,
    Parametrise(u16, u8),
}
pub fn compile(ctl_graph: &CtlGraph) -> ByteCode {
    let mut code = Vec::new();
    let mut node_to_device = BTreeMap::new();
    let mut devices = FuseBox::new();

    let mut output_params: OutputMap = BTreeMap::new();

    for (&nid, (_, params)) in ctl_graph.graph.iter() {
        for (pid, &psid) in params
            .iter()
            .enumerate()
            .map(|(pid, psid)| (pid as u8, psid))
        {
            if let Some(psid) = psid {
                output_params.entry(psid).or_default().push((nid, pid))
            }
        }
    }
    for (nid, params) in output_params.into_iter().rev() {
        match ctl_graph.graph[&nid].0 {
            DeviceKind::MidiControl => continue,
            DeviceKind::Audio(dd) => {
                let d = (DEVICES[dd].make)(&mut devices);
                node_to_device.insert(nid, d);
            }
            DeviceKind::Control => {
                let d = {
                    let i = devices.len();
                    devices.push(Control(0.0));
                    i
                };
                node_to_device.insert(nid, d);
            }
            DeviceKind::Output => {
                let d = devices.len();
                devices.push(Output(0.0));
                node_to_device.insert(nid, d);
                code.push(Op::Output);
                continue;
            }
        }
        code.push(Op::Sample(nid));
        for (puid, pid) in params {
            code.push(Op::Parametrise(puid, pid));
        }
    }

    ByteCode {
        devices,
        node_to_device,
        code,
        sample: 0.0,
    }
}
