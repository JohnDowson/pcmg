use crate::devices::Device;
use fusebox::FuseBox;
use std::collections::BTreeMap;

use super::{
    CtlGraph,
    DeviceId,
};

type NodeToDevice = BTreeMap<DeviceId, usize>;
type OutputMap = BTreeMap<(DeviceId, u8), Vec<(DeviceId, u8)>>;

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
    pub fn update_param(&mut self, _pid @ (dev, param): (DeviceId, u8), value: f32) {
        let d = self.node_to_device[&dev];
        let d = self.devices.get_mut(d).expect("No such device");
        d.set_param_indexed(param, value)
    }

    pub fn sample(&mut self) -> f32 {
        for op in &self.code {
            match op {
                Op::Sample(d, oid) => {
                    self.sample = self.devices[*d as usize].get_output_indexed(*oid);
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
    Sample(u16, u8),
    Output,
    Parametrise(u16, u8),
}
pub fn compile(ctl_graph: &CtlGraph) -> ByteCode {
    let mut code = Vec::new();
    let mut node_to_device = BTreeMap::new();
    let mut devices = FuseBox::new();
    let mut output_params: OutputMap = BTreeMap::new();

    for (&nid, &(d, params)) in &ctl_graph.graph {
        let d = d.make()(&mut devices);
        node_to_device.insert(nid, d);

        // map previous to next
        for (pid, &psid) in params
            .iter()
            .enumerate()
            .map(|(pid, psid)| (pid as u8, psid))
        {
            if let Some(psid) = psid {
                let params = output_params.entry(psid).or_default();
                params.push((nid, pid))
            }
        }
    }

    for ((nid, oid), params) in output_params.into_iter().rev() {
        code.push(Op::Sample(node_to_device[&nid] as u16, oid));
        for (puid, pid) in params {
            code.push(Op::Parametrise(node_to_device[&puid] as u16, pid));
        }
    }

    code.push(Op::Output);

    ByteCode {
        devices,
        node_to_device,
        code,
        sample: 0.0,
    }
}
