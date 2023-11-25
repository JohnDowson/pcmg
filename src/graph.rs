use crate::{
    build_midi_in,
    devices::{FILTER_DESCRIPTIONS, MIXER_DESCRIPTIONS, SYNTH_DESCRIPTIONS},
    widgets::{knob::SimpleKnob, scope::SampleQueue},
    STQueue,
};
#[cfg(target_arch = "wasm32")]
use cpal::traits::StreamTrait;
use cpal::Stream;
use eframe::{
    egui::{self, DragValue},
    epaint::Color32,
};
use egui_node_graph::{
    DataTypeTrait, Graph, GraphEditorState, InputParamKind, NodeDataTrait, NodeId, NodeResponse,
    NodeTemplateIter, NodeTemplateTrait, UserResponseTrait, WidgetValueTrait,
};
use midir::MidiInputConnection;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use wmidi::MidiMessage;

type MetaGraph = (
    BTreeMap<NodeId, u16>,
    BTreeSet<u16>,
    BTreeMap<u16, (NodeKind, [Option<u16>; 16])>,
);

pub enum UiMessage {
    Rebuild(Arc<MetaGraph>),
    KnobChanged(u16, f32),
    MidiPortChanged(STQueue<(u64, MidiMessage<'static>)>),
}

#[derive(Default)]
pub struct PcmgGraphState {
    output: Option<NodeId>,
    knobs: BTreeMap<NodeId, SimpleKnob>,
}

pub struct NodeTemplates;
impl NodeTemplateIter for NodeTemplates {
    type Item = NodeKind;

    fn all_kinds(&self) -> Vec<Self::Item> {
        SYNTH_DESCRIPTIONS
            .iter()
            .enumerate()
            .map(|(i, _)| NodeKind::Synth(i))
            .chain(
                FILTER_DESCRIPTIONS
                    .iter()
                    .enumerate()
                    .map(|(i, _)| NodeKind::Filter(i)),
            )
            .chain(
                MIXER_DESCRIPTIONS
                    .iter()
                    .enumerate()
                    .map(|(i, _)| NodeKind::Mixer(i)),
            )
            .chain(std::iter::once(NodeKind::MidiControl))
            .chain(std::iter::once(NodeKind::Knob))
            .chain(std::iter::once(NodeKind::Output))
            .collect()
    }
}

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub enum NodeKind {
    #[default]
    Output,
    Knob,
    MidiControl,
    Synth(usize),
    Filter(usize),
    Mixer(usize),
}

impl Eq for NodeKind {}

impl NodeDataTrait for NodeKind {
    type Response = PcmgNodeResponse;

    type UserState = PcmgGraphState;

    type DataType = Scalar;

    type ValueType = NodeKind;

    fn bottom_ui(
        &self,
        _ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<Self::Response, Self>>
    where
        Self::Response: UserResponseTrait,
    {
        vec![]
    }

    fn can_delete(
        &self,
        node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
    ) -> bool {
        user_state.output.map(|nid| nid != node_id).unwrap_or(true)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Scalar;
impl DataTypeTrait<PcmgGraphState> for Scalar {
    fn data_type_color(&self, _user_state: &mut PcmgGraphState) -> Color32 {
        Color32::from_rgb(180, 100, 0)
    }

    fn name(&self) -> Cow<str> {
        Cow::Borrowed("Scalar")
    }
}

impl NodeTemplateTrait for NodeKind {
    type NodeData = Self;
    type DataType = Scalar;
    type ValueType = NodeKind;
    type UserState = PcmgGraphState;
    type CategoryType = ();

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> std::borrow::Cow<str> {
        match self {
            NodeKind::Output => Cow::Borrowed("Output"),
            NodeKind::Knob => Cow::Borrowed("Knob"),
            NodeKind::MidiControl => Cow::Borrowed("Midi Control"),
            NodeKind::Synth(id) => Cow::Borrowed(SYNTH_DESCRIPTIONS[*id].name),
            NodeKind::Filter(id) => Cow::Borrowed(FILTER_DESCRIPTIONS[*id].name),
            NodeKind::Mixer(id) => Cow::Borrowed(MIXER_DESCRIPTIONS[*id].name),
        }
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        *self
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        let params = match self {
            NodeKind::Output => {
                if user_state.output.is_some() {
                    return;
                }
                graph.add_input_param(
                    node_id,
                    "Output".into(),
                    Scalar,
                    *self,
                    InputParamKind::ConnectionOnly,
                    true,
                );
                user_state.output = Some(node_id);
                return;
            }
            NodeKind::Knob => {
                user_state.knobs.insert(
                    node_id,
                    SimpleKnob::new((0.0, 2200.0), (0.0, 360.0), 0.0, 0.05, 24.0),
                );
                graph.add_input_param(
                    node_id,
                    "Control".into(),
                    Scalar,
                    *self,
                    InputParamKind::ConstantOnly,
                    true,
                );
                &[]
            }
            NodeKind::MidiControl => &[],
            NodeKind::Synth(id) => SYNTH_DESCRIPTIONS[*id].params,
            NodeKind::Filter(id) => FILTER_DESCRIPTIONS[*id].params,
            NodeKind::Mixer(id) => MIXER_DESCRIPTIONS[*id].params,
        };

        for param in params {
            graph.add_input_param(
                node_id,
                param.name.into(),
                Scalar,
                *self,
                InputParamKind::ConnectionOnly,
                true,
            );
        }
        graph.add_output_param(node_id, "Output".into(), Scalar);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PcmgNodeResponse {
    NewOutput,
    KnobChanged(NodeId, f32),
}

impl UserResponseTrait for PcmgNodeResponse {}

impl WidgetValueTrait for NodeKind {
    type Response = PcmgNodeResponse;

    type UserState = PcmgGraphState;

    type NodeData = NodeKind;

    fn value_widget(
        &mut self,
        param_name: &str,
        node_id: NodeId,
        ui: &mut egui::Ui,
        user_state: &mut Self::UserState,
        _node_data: &Self::NodeData,
    ) -> Vec<Self::Response> {
        let is_output = user_state.output.map(|nid| nid == node_id).unwrap_or(false);
        match self {
            NodeKind::Knob => {
                let knob = ui
                    .vertical(|ui| {
                        let knob = user_state.knobs.get_mut(&node_id).unwrap();
                        ui.horizontal(|ui| {
                            ui.add(DragValue::new(&mut knob.value_range.start));
                            ui.add(DragValue::new(&mut knob.value_range.end));
                        });
                        ui.add(knob)
                    })
                    .inner;
                let knob_widget = user_state.knobs.get(&node_id).unwrap();
                if knob.changed() {
                    vec![PcmgNodeResponse::KnobChanged(node_id, knob_widget.value)]
                } else {
                    vec![]
                }
            }
            NodeKind::Output if !is_output => vec![PcmgNodeResponse::NewOutput],
            _ => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                });
                vec![]
            }
        }
    }
}

type PcmgGraph = Graph<NodeKind, Scalar, NodeKind>;
type PcmgGraphEditorState = GraphEditorState<NodeKind, Scalar, NodeKind, NodeKind, PcmgGraphState>;

pub struct PcmgNodeGraph {
    editor: PcmgGraphEditorState,
    last_synth_graph: Arc<MetaGraph>,
    ui_tx: STQueue<UiMessage>,
    state: PcmgGraphState,

    ports: Vec<String>,
    port: Option<usize>,
    midi_conn: Option<MidiInputConnection<()>>,

    #[allow(dead_code)]
    stream: Stream,
    samples: SampleQueue,
}

impl PcmgNodeGraph {
    pub fn new(
        ui_tx: STQueue<UiMessage>,
        stream: Stream,
        ports: Vec<String>,
        midi_conn: Option<MidiInputConnection<()>>,
        samples: SampleQueue,
    ) -> Self {
        Self {
            editor: Default::default(),
            last_synth_graph: Default::default(),
            ui_tx,
            state: Default::default(),
            ports,
            port: None,
            midi_conn,
            stream,
            samples,
        }
    }
}

impl eframe::App for PcmgNodeGraph {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let port = self.port;
        egui::TopBottomPanel::top("top-bar").show(ctx, |ui| {
            let selected_text = if let Some(p) = self.port {
                &self.ports[p]
            } else {
                "None"
            };
            egui::ComboBox::from_label("MIDI in")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (i, port) in self.ports.iter().enumerate() {
                        ui.selectable_value(&mut self.port, Some(i), port);
                    }
                });

            #[cfg(target_arch = "wasm32")]
            if ui.add(egui::Button::new("Start sound")).clicked() {
                self.stream.play().unwrap();
            }
        });

        egui::TopBottomPanel::bottom("scope")
            .resizable(true)
            .show(ctx, |ui| {
                use egui::plot::{Line, Plot, PlotPoints};
                let sin: PlotPoints = self
                    .samples
                    .get()
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(i, s)| [i as f64, s as f64])
                    .collect();
                let line = Line::new(sin);
                Plot::new("Waveform")
                    .include_y(1.0)
                    .include_y(-1.0)
                    // .view_aspect(2.0)
                    .show(ui, |plot_ui| plot_ui.line(line));

                ctx.request_repaint();
            });

        if port != self.port {
            if let Some(p) = self.port {
                let midi_evs = STQueue::new();
                let (_, mut conn) = build_midi_in(midi_evs.clone(), p).unwrap();
                std::mem::swap(&mut self.midi_conn, &mut conn);
                conn.map(|c| c.close());
                self.ui_tx.put(UiMessage::MidiPortChanged(midi_evs));
            } else {
                self.midi_conn.take().map(|c| c.close());
            }
        }

        let graph_resp = egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.editor
                    .draw_graph_editor(ui, NodeTemplates, &mut self.state, vec![])
            })
            .inner;

        let mut rebuild = false;
        for node_resp in graph_resp.node_responses {
            match node_resp {
                NodeResponse::ConnectEventEnded { .. } => {
                    rebuild = true;
                }
                NodeResponse::DeleteNodeFull { node_id, .. } => {
                    self.state.knobs.remove(&node_id);
                    rebuild = true;
                }
                NodeResponse::DisconnectEvent { .. } => {
                    rebuild = true;
                }
                NodeResponse::User(resp) => match resp {
                    PcmgNodeResponse::NewOutput => {
                        rebuild = true;
                    }
                    PcmgNodeResponse::KnobChanged(node_id, value) => {
                        let Some(nid) = self.last_synth_graph.0.get(&node_id) else {
                            continue;
                        };

                        self.ui_tx.put(UiMessage::KnobChanged(*nid, value));
                    }
                },
                _ => {}
            }
        }
        if let Some(output) = self.state.output {
            if rebuild {
                let synth_graph = walk_build(output, &self.editor.graph);
                if *self.last_synth_graph != synth_graph {
                    self.last_synth_graph = Arc::new(synth_graph);
                    self.ui_tx
                        .put(UiMessage::Rebuild(Arc::clone(&self.last_synth_graph)));
                    for (node_id, knob) in &self.state.knobs {
                        let Some(nid) = self.last_synth_graph.0.get(node_id) else {
                            continue;
                        };
                        self.ui_tx.put(UiMessage::KnobChanged(*nid, knob.value));
                    }
                }
            }
        }
    }
}

fn prev_nodes(curr: NodeId, graph: &PcmgGraph) -> impl Iterator<Item = (usize, NodeId)> + '_ {
    let curr = graph.nodes.get(curr).unwrap();
    curr.inputs
        .iter()
        .enumerate()
        .filter_map(|(i, (_, input))| {
            graph.connections.get(*input).map(|output| {
                let output = graph.outputs.get(*output).unwrap();
                (i, output.node)
            })
        })
}

fn walk_build(output: NodeId, graph: &PcmgGraph) -> MetaGraph {
    let mut stack = BTreeMap::new();
    let mut midi_ins = BTreeSet::new();
    let mut mapping = BTreeMap::new();
    depth_first(
        output,
        graph,
        &mut stack,
        &mut midi_ins,
        &mut mapping,
        &mut 0,
    );
    (mapping, midi_ins, stack)
}

fn depth_first(
    node: NodeId,
    graph: &PcmgGraph,
    stack: &mut BTreeMap<u16, (NodeKind, [Option<u16>; 16])>,
    midi_ins: &mut BTreeSet<u16>,
    mapping: &mut BTreeMap<NodeId, u16>,
    counter: &mut u16,
) -> u16 {
    let this = *counter;
    let kind = graph.nodes.get(node).unwrap().user_data;
    *counter += 1;
    let mut inputs = [None; 16];
    for (i, node) in prev_nodes(node, graph) {
        inputs[i] = Some(depth_first(node, graph, stack, midi_ins, mapping, counter));
    }

    *mapping.entry(node).or_insert_with(|| {
        stack.insert(this, (kind, inputs));
        if matches!(kind, NodeKind::MidiControl) {
            midi_ins.insert(this);
        }
        this
    })
}
