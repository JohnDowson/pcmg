use crate::devices::Device;
use fusebox::FuseBox;
use std::collections::{
    BTreeMap,
    BTreeSet,
    VecDeque,
};

use super::{
    CtlGraph,
    DeviceId,
};

type NodeToDevice = BTreeMap<DeviceId, usize>;
type OutputMap = BTreeMap<usize, ((DeviceId, u8), Vec<(DeviceId, u8)>)>;

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
    pub fn update_param(&mut self, pid @ (dev, param): (DeviceId, u8), value: f32) {
        let d = if let Some(d) = self.node_to_device.get(&dev) {
            *d
        } else {
            dbg!(pid);
            return;
        };
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
pub fn compile(ctl_graph: &CtlGraph, sample_rate: f32) -> ByteCode {
    dbg!(ctl_graph);
    let mut graph = ctl_graph.graph.clone();
    let mut code = VecDeque::new();
    let mut node_to_device = BTreeMap::new();
    let mut devices = FuseBox::new();
    let mut output_params: OutputMap = BTreeMap::new();

    let mut last = ctl_graph.end;
    let mut counter = 0;

    let mut prevs = BTreeSet::new();
    while let Some((dev, params)) = graph.remove(&last) {
        let device_idx = dev.make()(&mut devices, sample_rate);
        node_to_device.insert(last, device_idx);

        for (pid, params) in params.into_iter().enumerate() {
            if let Some((source_did, source_pid)) = params {
                prevs.insert(source_did);

                let (_, to_parametrise) = output_params
                    .entry(counter)
                    .or_insert(((source_did, source_pid), Vec::new()));
                counter += 1;

                to_parametrise.push((last, pid as u8));
            }
        }
        if let Some(new) = prevs.pop_first() {
            last = new;
        } else {
            break;
        }
    }

    for ((nid, oid), params) in output_params.into_values().rev() {
        code.push_back(Op::Sample(node_to_device[&nid] as u16, oid));
        for (puid, pid) in params {
            code.push_back(Op::Parametrise(node_to_device[&puid] as u16, pid));
        }
    }

    code.push_back(Op::Output);

    let code = code.into();
    dbg! {ByteCode {
        devices,
        node_to_device,
        code,
        sample: 0.0,
    }}
}
