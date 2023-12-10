use std::{
    collections::BTreeMap,
    ops::{
        Index,
        IndexMut,
    },
};

use slotmap::{
    new_key_type,
    SecondaryMap,
    SlotMap,
};

use crate::devices::description::DeviceKind;

use self::modules::Module;

pub mod compiled;
pub mod modules;

new_key_type! {
    pub struct ModuleId;
    pub struct DeviceId;
    pub struct VisualId;

    pub struct InputId;
    pub struct OutputId;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Connector {
    /// Connected to an input of a device
    In(InputId),
    /// Connected to an output of a device
    Out(OutputId),
}

#[derive(Default, Debug)]
pub struct Graph {
    pub modules: SlotMap<ModuleId, Module>,
    pub devices: SlotMap<DeviceId, DeviceKind>,

    /// What device a given input belongs o
    pub ins: SlotMap<InputId, (DeviceId, u8)>,
    /// What device a given output belongs to
    pub outs: SlotMap<OutputId, (DeviceId, u8)>,
    /// What ins a given device has
    pub dev_ins: SecondaryMap<DeviceId, Vec<InputId>>,
    /// What outs a given device has
    pub dev_outs: SecondaryMap<DeviceId, Vec<OutputId>>,
    /// Connections linking node's inputs back to outputs
    pub cables: SecondaryMap<InputId, OutputId>,
}

type CtlGraphGraph = BTreeMap<DeviceId, (DeviceKind, [Option<(DeviceId, u8)>; 16])>;

#[derive(Debug, Default)]
pub struct CtlGraph {
    pub end: DeviceId,
    pub dev_map: BTreeMap<Connector, (DeviceId, u8)>,
    pub midis: SecondaryMap<OutputId, (DeviceId, u8)>,
    graph: CtlGraphGraph,
}

struct Walker {
    dev_map: BTreeMap<Connector, (DeviceId, u8)>,
    midis: SecondaryMap<OutputId, (DeviceId, u8)>,
    /// from node closer to output backwards
    graph: CtlGraphGraph,
}

impl Walker {
    fn walk(to: InputId, graph: &Graph) -> CtlGraph {
        let end = graph.ins[to].0;
        let mut this = Self {
            dev_map: Default::default(),
            midis: Default::default(),
            graph: Default::default(),
        };

        this.walk_input(to, graph);

        let Walker {
            dev_map,
            midis,
            graph,
        } = this;

        CtlGraph {
            end,
            dev_map,
            midis,
            graph,
        }
    }

    fn walk_input(&mut self, input: InputId, graph: &Graph) {
        let (dev, param) = graph[input];
        let dev_desc = graph.devices[dev];
        println!("Walking input {param} for {dev_desc:?}");

        self.dev_map.insert(Connector::In(input), (dev, param));

        let prev = graph.cables.get(input).copied().map(|out| graph[out]);
        if let Some((prev_dev, _)) = prev {
            // for each of this device's previous devices' outputs
            let prevs = graph
                .dev_outs
                .get(prev_dev)
                .map(|outs| &**outs)
                .unwrap_or(&[])
                .iter()
                .copied()
                .map(|out| (out, graph.outs.get(out).copied()));

            for (output, _) in prevs {
                let r = self.walk_output(output, graph);

                let (_, params) = self.graph.entry(dev).or_insert((dev_desc, [None; 16]));
                params[param as usize] = Some(r);
            }
        }
    }

    fn walk_output(&mut self, output: OutputId, graph: &Graph) -> (DeviceId, u8) {
        let (dev, param) = graph[output];
        let dev_desc = graph.devices[dev];
        println!("Walking output {param} for {dev_desc:?}");

        if matches!(dev_desc, DeviceKind::MidiControl) {
            self.midis.insert(output, (dev, param));
        }
        let (_, _) = self.graph.entry(dev).or_insert((dev_desc, [None; 16]));

        self.dev_map.insert(Connector::Out(output), (dev, param));

        // for each of this device's inputs
        let prevs = graph
            .dev_ins
            .get(dev)
            .map(|ins| &**ins)
            .unwrap_or(&[])
            .iter()
            .copied();

        for input in prevs {
            self.walk_input(input, graph);
        }
        (dev, param)
    }
}

impl Graph {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn remove_module(&mut self, mid: ModuleId) {
        let (mut removed_ins, mut removed_outs) = (Vec::new(), Vec::new());

        let module = self.modules.remove(mid).unwrap();
        for did in module.devices {
            self.ins.retain(|i, (dev, _)| {
                if *dev != did {
                    removed_ins.push(i);

                    true
                } else {
                    false
                }
            });
            self.outs.retain(|o, (dev, _)| {
                if *dev != did {
                    removed_outs.push(o);

                    true
                } else {
                    false
                }
            });
        }
        for i in removed_ins {
            self.cables.remove(i);
        }

        for o in removed_outs {
            self.cables.retain(|_, co| o != *co);
        }
    }

    pub fn walk_to(&self, end: InputId) -> CtlGraph {
        Walker::walk(end, self)
    }
}

impl Index<InputId> for Graph {
    type Output = (DeviceId, u8);

    fn index(&self, index: InputId) -> &Self::Output {
        &self.ins[index]
    }
}

impl Index<OutputId> for Graph {
    type Output = (DeviceId, u8);

    fn index(&self, index: OutputId) -> &Self::Output {
        &self.outs[index]
    }
}

impl Index<ModuleId> for Graph {
    type Output = Module;

    fn index(&self, index: ModuleId) -> &Self::Output {
        self.modules.get(index).unwrap()
    }
}

impl IndexMut<ModuleId> for Graph {
    fn index_mut(&mut self, index: ModuleId) -> &mut Self::Output {
        self.modules.get_mut(index).unwrap()
    }
}
