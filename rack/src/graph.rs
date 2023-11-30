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

use crate::{
    container::module::Module,
    devices::DeviceKind,
};

new_key_type! {
   pub struct InputId;
   pub struct OutputId;
   pub struct ModuleId;
}

pub struct InputNode {}
pub struct OutputNode {}

#[derive(Default)]
pub struct Graph {
    pub modules: SlotMap<ModuleId, Module>,

    /// What module a given input belongs to
    pub ins: SlotMap<InputId, ModuleId>,
    /// What module a given output belongs to
    pub outs: SlotMap<OutputId, ModuleId>,
    /// Connections linking modules inputs back to previuos modules outputs
    pub cables: SecondaryMap<InputId, OutputId>,
}

pub struct CtlGraph {
    pub dev_map: SecondaryMap<OutputId, u16>,
    pub graph: BTreeMap<u16, (DeviceKind, [Option<u16>; 16])>,
}

struct Walker {
    counter: u16,
    dev_map: SecondaryMap<OutputId, u16>,
    graph: BTreeMap<u16, (DeviceKind, [Option<u16>; 16])>,
}

impl Walker {
    fn walk(out: OutputId, graph: &Graph) -> CtlGraph {
        let mut this = Self {
            counter: 0,
            dev_map: Default::default(),
            graph: Default::default(),
        };

        this.walk_build(out, graph);
        let Walker {
            counter: _,
            dev_map,
            graph,
        } = this;
        CtlGraph { dev_map, graph }
    }

    fn walk_build(&mut self, out: OutputId, graph: &Graph) -> u16 {
        let this = self.counter;

        self.counter += 1;
        let curr = graph[out];
        let module = &graph[curr];
        let kind = module.dev_desc.kind;
        let prevs = module
            .ins
            .values()
            .enumerate()
            .filter_map(|(i, inp)| graph.cables.get(*inp).map(|&out| (i, out)));

        let mut inputs = [None; 16];
        for (i, node) in prevs {
            inputs[i] = Some(self.walk_build(node, graph));
        }

        *self.dev_map.entry(out).unwrap().or_insert_with(|| {
            self.graph.insert(this, (kind, inputs));
            // if matches!(kind, NodeKind::MidiControl) {
            //     midi_ins.insert(this);
            // }
            this
        })
    }
}

impl Graph {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn remove_module(&mut self, id: ModuleId) -> Option<Module> {
        let (mut removed_ins, mut removed_outs) = (Vec::new(), Vec::new());
        self.ins.retain(|i, m| {
            if *m != id {
                removed_ins.push(i);

                true
            } else {
                false
            }
        });
        self.outs.retain(|o, m| {
            if *m != id {
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

        self.modules.remove(id)
    }

    pub fn walk_to(&self, end: InputId) -> CtlGraph {
        let out = self[end];

        Walker::walk(out, self)
    }
}

impl Index<InputId> for Graph {
    type Output = OutputId;

    fn index(&self, index: InputId) -> &Self::Output {
        self.cables.get(index).unwrap()
    }
}

impl Index<OutputId> for Graph {
    type Output = ModuleId;

    fn index(&self, index: OutputId) -> &Self::Output {
        self.outs.get(index).unwrap()
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
