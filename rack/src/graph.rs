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
    pub nodes: SlotMap<NodeId, Node>,
    pub devices: SlotMap<DeviceId, DeviceKind>,

    /// What node a given input belongs to
    pub ins: SlotMap<InputId, NodeId>,
    /// What node a given output belongs to
    pub outs: SlotMap<OutputId, NodeId>,
    /// Connections linking node's inputs back to outputs
    pub cables: SecondaryMap<InputId, OutputId>,
}

#[derive(Debug, Default)]
pub struct CtlGraph {
    pub dev_map: BTreeMap<Connector, (DeviceId, u16)>,
    pub midis: SecondaryMap<OutputId, (DeviceId, u16)>,
    graph: BTreeMap<DeviceId, (DeviceKind, [Option<DeviceId>; 16])>,
}

struct Walker {
    dev_map: BTreeMap<Connector, (DeviceId, u16)>,
    midis: SecondaryMap<OutputId, (DeviceId, u16)>,
    /// from node closer to output backwards
    graph: BTreeMap<DeviceId, (DeviceKind, [Option<DeviceId>; 16])>,
}

impl Walker {
    fn walk(to: InputId, graph: &Graph) -> CtlGraph {
        let mut this = Self {
            dev_map: Default::default(),
            midis: Default::default(),
            graph: Default::default(),
        };

        dbg!(graph);
        this.walk_input(to, graph);

        let Walker {
            dev_map,
            midis,
            graph,
        } = this;

        dbg!(&dev_map);
        dbg!(&graph);
        CtlGraph {
            dev_map,
            midis,
            graph,
        }
    }

    fn walk_input(&mut self, input: InputId, graph: &Graph) -> Option<DeviceId> {
        let this_node = &graph[graph[input]];
        let (dev, param) = this_node.input_to_param[input];
        let dev_desc = graph.devices[dev];
        self.dev_map
            .insert(Connector::In(input), (dev, param as u16));

        // let prev_out = graph.cables.get(input).copied().map(|prev_out| {
        //     self.walk_output(prev_out, graph);
        //     prev_out
        // });
        let r = graph
            .cables
            .get(input)
            .copied()
            .map(|prev_out| self.walk_output(prev_out, graph));

        let (_, params) = self.graph.entry(dev).or_insert((dev_desc, [None; 16]));

        params[param] = r;
        r
        // prev_out.map(|o| graph[graph[o]].output_to_param[o].0)
    }

    fn walk_output(&mut self, output: OutputId, graph: &Graph) -> DeviceId {
        let this_node = &graph[graph[output]];
        let (dev, param) = this_node.output_to_param[output];
        let dev_desc = graph.devices[dev];

        self.dev_map
            .insert(Connector::Out(output), (dev, param as u16));

        // for each of this device's inputs
        let prevs = this_node
            .input_to_param
            .iter()
            .filter_map(|(inp, &(d, p))| (d == dev).then_some((inp, p)));

        for (input, _param) in prevs {
            let r = self.walk_input(input, graph);

            let (_, params) = self.graph.entry(dev).or_insert((dev_desc, [None; 16]));

            params[param] = r;
        }

        dev
    }

    // fn walk_build(&mut self, input: InputId, graph: &Graph) -> u16 {
    //     let this = self.counter;
    //     if let Some((id, _)) = self.dev_map.get(&Connector::In(input)) {
    //         return *id;
    //     }
    //     dbg!(input);

    //     self.counter += 1;

    //     let mut params = [None; 16];
    //     let node = graph[input];
    //     let node = &graph[node];

    //     for (out, &(dev, param)) in &node.output_to_param {
    //         self.dev_map
    //             .insert(Connector::Out(out), (this, param as u16));
    //         let dev_desc = node.devices[dev];
    //         if matches!(dev_desc, DeviceKind::MidiControl) {
    //             self.midis.insert(out, (this, param as u16));
    //         }
    //     }

    //     let (dev, param) = node.input_to_param[input];

    //     // umm actually:
    //     // knobs are connected to inputs OR outputs
    //     // ie: CtlDev has knob on it's output
    //     // ie2: Osc has knob on it's input
    //     // Umm actually no it dont, we won't have anything to sample for that input
    //     // Should we even do that, sampling?
    //     // Recalculating params is somewhat costly so doing that for each sample is kinda yuck

    //     // for each input on this input's node

    //     for (inp, pi) in prevs {
    //         params[pi] = Some(self.walk_build(inp, graph));
    //     }

    //     self.dev_map
    //         .entry(Connector::In(input))
    //         .or_insert_with(|| {
    //             let dev_desc = node.devices[dev];

    //             self.graph.insert(this, (dev_desc, params));

    //             (this, param as u16)
    //         })
    //         .0
    // }
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
    type Output = NodeId;

    fn index(&self, index: InputId) -> &Self::Output {
        &self.ins[index]
    }
}

impl Index<OutputId> for Graph {
    type Output = NodeId;

    fn index(&self, index: OutputId) -> &Self::Output {
        &self.outs[index]
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
