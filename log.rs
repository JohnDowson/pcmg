[rack/src/graph.rs:73] graph = Graph {
    modules: SlotMap {
        slots: [
            Slot {
                version: 0,
                next_free: 0,
            },
            Slot {
                version: 1,
                value: Module {
                    size: U1,
                    node: NodeId(
                        1v1,
                    ),
                    visuals_count: 1,
                    values: SecondaryMap {
                        slots: [
                            Vacant,
                            Occupied {
                                value: In(
                                    InputId(
                                        1v1,
                                    ),
                                ),
                                version: 1,
                            },
                        ],
                        num_elems: 1,
                        _k: PhantomData<fn(rack::graph::VisualId) -> rack::graph::VisualId>,
                    },
                    ins: SecondaryMap {
                        slots: [
                            Vacant,
                            Occupied {
                                value: VisualId(
                                    1v1,
                                ),
                                version: 1,
                            },
                        ],
                        num_elems: 1,
                        _k: PhantomData<fn(rack::graph::InputId) -> rack::graph::InputId>,
                    },
                    outs: SecondaryMap {
                        slots: [
                            Vacant,
                        ],
                        num_elems: 0,
                        _k: PhantomData<fn(rack::graph::OutputId) -> rack::graph::OutputId>,
                    },
                },
            },
            Slot {
                version: 1,
                value: Module {
                    size: U2,
                    node: NodeId(
                        2v1,
                    ),
                    visuals_count: 3,
                    values: SecondaryMap {
                        slots: [
                            Vacant,
                            Occupied {
                                value: In(
                                    InputId(
                                        2v1,
                                    ),
                                ),
                                version: 1,
                            },
                            Occupied {
                                value: In(
                                    InputId(
                                        3v1,
                                    ),
                                ),
                                version: 1,
                            },
                            Occupied {
                                value: Out(
                                    OutputId(
                                        1v1,
                                    ),
                                ),
                                version: 1,
                            },
                        ],
                        num_elems: 3,
                        _k: PhantomData<fn(rack::graph::VisualId) -> rack::graph::VisualId>,
                    },
                    ins: SecondaryMap {
                        slots: [
                            Vacant,
                            Vacant,
                            Occupied {
                                value: VisualId(
                                    1v1,
                                ),
                                version: 1,
                            },
                            Occupied {
                                value: VisualId(
                                    2v1,
                                ),
                                version: 1,
                            },
                        ],
                        num_elems: 2,
                        _k: PhantomData<fn(rack::graph::InputId) -> rack::graph::InputId>,
                    },
                    outs: SecondaryMap {
                        slots: [
                            Vacant,
                            Occupied {
                                value: VisualId(
                                    3v1,
                                ),
                                version: 1,
                            },
                        ],
                        num_elems: 1,
                        _k: PhantomData<fn(rack::graph::OutputId) -> rack::graph::OutputId>,
                    },
                },
            },
        ],
        free_head: 3,
        num_elems: 2,
        _k: PhantomData<fn(rack::graph::ModuleId) -> rack::graph::ModuleId>,
    },
    nodes: SlotMap {
        slots: [
            Slot {
                version: 0,
                next_free: 0,
            },
            Slot {
                version: 1,
                value: Node {
                    devices: SlotMap {
                        slots: [
                            Slot {
                                version: 0,
                                next_free: 0,
                            },
                            Slot {
                                version: 1,
                                value: Output,
                            },
                        ],
                        free_head: 2,
                        num_elems: 1,
                        _k: PhantomData<fn(rack::graph::DeviceId) -> rack::graph::DeviceId>,
                    },
                    input_to_param: SecondaryMap {
                        slots: [
                            Vacant,
                            Occupied {
                                value: (
                                    DeviceId(
                                        1v1,
                                    ),
                                    0,
                                ),
                                version: 1,
                            },
                        ],
                        num_elems: 1,
                        _k: PhantomData<fn(rack::graph::InputId) -> rack::graph::InputId>,
                    },
                    output_to_param: SecondaryMap {
                        slots: [
                            Vacant,
                        ],
                        num_elems: 0,
                        _k: PhantomData<fn(rack::graph::OutputId) -> rack::graph::OutputId>,
                    },
                },
            },
            Slot {
                version: 1,
                value: Node {
                    devices: SlotMap {
                        slots: [
                            Slot {
                                version: 0,
                                next_free: 0,
                            },
                            Slot {
                                version: 1,
                                value: Audio(
                                    0,
                                ),
                            },
                        ],
                        free_head: 2,
                        num_elems: 1,
                        _k: PhantomData<fn(rack::graph::DeviceId) -> rack::graph::DeviceId>,
                    },
                    input_to_param: SecondaryMap {
                        slots: [
                            Vacant,
                            Vacant,
                            Occupied {
                                value: (
                                    DeviceId(
                                        1v1,
                                    ),
                                    0,
                                ),
                                version: 1,
                            },
                            Occupied {
                                value: (
                                    DeviceId(
                                        1v1,
                                    ),
                                    1,
                                ),
                                version: 1,
                            },
                        ],
                        num_elems: 2,
                        _k: PhantomData<fn(rack::graph::InputId) -> rack::graph::InputId>,
                    },
                    output_to_param: SecondaryMap {
                        slots: [
                            Vacant,
                            Occupied {
                                value: (
                                    DeviceId(
                                        1v1,
                                    ),
                                    2,
                                ),
                                version: 1,
                            },
                        ],
                        num_elems: 1,
                        _k: PhantomData<fn(rack::graph::OutputId) -> rack::graph::OutputId>,
                    },
                },
            },
        ],
        free_head: 3,
        num_elems: 2,
        _k: PhantomData<fn(rack::graph::NodeId) -> rack::graph::NodeId>,
    },
    ins: SlotMap {
        slots: [
            Slot {
                version: 0,
                next_free: 0,
            },
            Slot {
                version: 1,
                value: NodeId(
                    1v1,
                ),
            },
            Slot {
                version: 1,
                value: NodeId(
                    2v1,
                ),
            },
            Slot {
                version: 1,
                value: NodeId(
                    2v1,
                ),
            },
        ],
        free_head: 4,
        num_elems: 3,
        _k: PhantomData<fn(rack::graph::InputId) -> rack::graph::InputId>,
    },
    outs: SlotMap {
        slots: [
            Slot {
                version: 0,
                next_free: 0,
            },
            Slot {
                version: 1,
                value: NodeId(
                    2v1,
                ),
            },
        ],
        free_head: 2,
        num_elems: 1,
        _k: PhantomData<fn(rack::graph::OutputId) -> rack::graph::OutputId>,
    },
    cables: SecondaryMap {
        slots: [
            Vacant,
            Occupied {
                value: OutputId(
                    1v1,
                ),
                version: 1,
            },
        ],
        num_elems: 1,
        _k: PhantomData<fn(rack::graph::InputId) -> rack::graph::InputId>,
    },
}
[rack/src/graph.rs:99] input = InputId(
    1v1,
)
[rack/src/graph.rs:99] input = InputId(
    2v1,
)
