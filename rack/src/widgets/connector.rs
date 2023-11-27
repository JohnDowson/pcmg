mod in_port;
mod out_port;

use std::{collections::BTreeMap, num::NonZeroU16, ops::Index};

pub use in_port::InPort;
pub use out_port::OutPort;
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::widget_description::{Wid, WidFull, WidgetKind};

new_key_type! {
   pub struct InputId;
   pub struct OutputId;
   pub struct ModuleId;
}

pub struct Cable {
    pub a_id: WidFull,
    pub b_id: WidFull,
}

pub struct InputNode {}
pub struct OutputNode {}

pub struct Slot {
    ins: Vec<(InputId, Wid)>,
}

#[derive(Default)]
pub struct Graph {
    pub modules: SlotMap<ModuleId, Slot>,
    pub ins: SlotMap<InputId, ModuleId>,
    pub outs: SlotMap<OutputId, ModuleId>,
    pub cables: SecondaryMap<InputId, OutputId>,
}

pub enum DeviceKind {
    Foo,
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

        let kind = DeviceKind::Foo;
        self.counter += 1;
        let curr = graph[out];
        let module = &graph[curr];
        let prevs = module
            .ins
            .iter()
            .enumerate()
            .filter_map(|(i, &(inp, _))| graph.cables.get(inp).map(|&out| (i, out)));

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
    type Output = Slot;

    fn index(&self, index: ModuleId) -> &Self::Output {
        self.modules.get(index).unwrap()
    }
}
