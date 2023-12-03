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

use self::{
    modules::Module,
    nodes::Node,
};

pub mod compiled;
pub mod modules;
pub mod nodes;

new_key_type! {
    pub struct ModuleId;
    pub struct NodeId;
    pub struct DeviceId;
    pub struct VisualId;

    pub struct InputId;
    pub struct OutputId;
}

#[derive(Clone, Copy, Debug)]
pub enum Connector {
    /// Connected to an input of a device
    In(InputId),
    /// Connected to an output of a device
    Out(OutputId),
}

#[derive(Default, Debug)]
pub struct Graph {
    pub modules: SlotMap<ModuleId, Module>,
    pub nodes: SlotMap<NodeId, Node>,

    /// What node a given input belongs to
    pub ins: SlotMap<InputId, NodeId>,
    /// What node a given output belongs to
    pub outs: SlotMap<OutputId, NodeId>,
    /// Connections linking node's inputs back to outputs
    pub cables: SecondaryMap<InputId, OutputId>,
}

#[derive(Debug, Default)]
pub struct CtlGraph {
    pub dev_map: SecondaryMap<InputId, (u16, u16)>,
    pub midis: SecondaryMap<InputId, (u16, u16)>,
    pub graph: BTreeMap<u16, (DeviceKind, [Option<u16>; 16])>,
}

struct Walker {
    counter: u16,
    dev_map: SecondaryMap<InputId, (u16, u16)>,
    midis: SecondaryMap<InputId, (u16, u16)>,
    graph: BTreeMap<u16, (DeviceKind, [Option<u16>; 16])>,
}

impl Walker {
    fn walk(to: InputId, graph: &Graph) -> CtlGraph {
        dbg!(graph);
        let mut this = Self {
            counter: 0,
            dev_map: Default::default(),
            midis: Default::default(),
            graph: Default::default(),
        };

        this.walk_build(to, graph);
        let Walker {
            counter: _,
            dev_map,
            midis,
            graph,
        } = this;
        CtlGraph {
            dev_map,
            midis,
            graph,
        }
    }

    fn walk_build(&mut self, input: InputId, graph: &Graph) -> u16 {
        let this = self.counter;
        self.counter += 1;

        let mut params = [None; 16];
        if let Some(output) = graph.cables.get(input).copied() {
            let node = graph[output];
            let node = &graph[node];
            let (dev, _) = node.output_to_param[output];
            let prevs = node
                .input_to_param
                .iter()
                .filter(|(_, (did, _))| *did == dev)
                .map(|(inp, (_, pi))| (inp, *pi));

            for (input, pi) in prevs {
                params[pi] = Some(self.walk_build(input, graph));
            }
        }
        let node = graph.ins[input];
        let node = &graph[node];
        self.dev_map
            .entry(input)
            .unwrap()
            .or_insert_with(|| {
                let (dev, param) = node.input_to_param[input];
                let dev_desc = node.devices[dev];

                self.graph.insert(this, (dev_desc, params));
                if matches!(dev_desc, DeviceKind::MidiControl) {
                    self.midis.insert(input, (this, 0));
                }
                (this, param as u16)
            })
            .0
    }
}

impl Graph {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn remove_module(&mut self, mid: ModuleId) -> Option<Module> {
        let nid = self[mid].node;
        let (mut removed_ins, mut removed_outs) = (Vec::new(), Vec::new());
        self.ins.retain(|i, m| {
            if *m != nid {
                removed_ins.push(i);

                true
            } else {
                false
            }
        });
        self.outs.retain(|o, m| {
            if *m != nid {
                removed_outs.push(o);

                true
            } else {
                false
            }
        });
        for i in removed_ins {
            self.cables.remove(i);
        }

        for o in removed_outs {
            self.cables.retain(|_, co| o != *co);
        }

        self.modules.remove(mid)
    }

    pub fn walk_to(&self, end: InputId) -> CtlGraph {
        Walker::walk(end, self)
    }
}

impl Index<InputId> for Graph {
    type Output = OutputId;

    fn index(&self, index: InputId) -> &Self::Output {
        self.cables.get(index).unwrap()
    }
}

impl Index<OutputId> for Graph {
    type Output = NodeId;

    fn index(&self, index: OutputId) -> &Self::Output {
        self.outs.get(index).unwrap()
    }
}

impl Index<NodeId> for Graph {
    type Output = Node;

    fn index(&self, index: NodeId) -> &Self::Output {
        self.nodes.get(index).unwrap()
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
