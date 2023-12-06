use eframe::{
    egui::Ui,
    epaint::{
        vec2,
        Color32,
        Rect,
    },
};

use egui::{
    Context,
    InnerResponse,
    Rounding,
    Stroke,
};
use emath::Pos2;
use itertools::Itertools;
use quadtree_rs::{
    area::AreaBuilder,
    point::Point,
    Quadtree,
};
use slotmap::SecondaryMap;
use wmidi::MidiMessage;

use crate::{
    devices::description::DeviceKind,
    graph::{
        modules::ModuleResponse,
        Connector,
        CtlGraph,
        Graph,
        InputId,
        ModuleId,
        OutputId,
    },
    widgets::connector::draw_catenary,
    STQueue,
};

use self::sizing::*;

pub mod sizing;

pub struct Stack {
    pub graph: Graph,
    end: Option<InputId>,
    attempting_connection: ConnAttempt,
    qt: Quadtree<u8, ModuleId>,
}

#[derive(Clone, Copy)]
enum ConnAttempt {
    None,
    In(InputId),
    Out(OutputId),
}

impl Stack {
    pub fn new() -> Self {
        Self {
            graph: Default::default(),
            end: None,
            // wires: Default::default(),
            attempting_connection: ConnAttempt::None,
            qt: Quadtree::new(2),
        }
    }

    pub fn with_module(&mut self, id: ModuleId) -> Option<ModuleId> {
        let sz = self.graph.modules[id].size;

        let mut ab = AreaBuilder::default();
        ab.dimensions(sz.size_in_units());

        let a = (0..self.qt.width())
            .cartesian_product(0..self.qt.height())
            .map(|(x, y)| {
                let (x, y) = (x as _, y as _);
                ab.anchor(Point { x, y });
                ab.build().unwrap()
            })
            .filter(|c| !self.qt.regions().any(|a| a.intersects(*c)))
            .min_by(|a, b| {
                let Point { x: ax, y: ay } = a.anchor();
                let Point { x: bx, y: by } = b.anchor();
                let (ax, ay, bx, by) = (ax as f32, ay as f32, bx as f32, by as f32);
                (ax.powi(2) + ay.powi(2))
                    .sqrt()
                    .total_cmp(&(bx.powi(2) + by.powi(2)).sqrt())
            });

        if let Some(a) = a {
            let devs = &self.graph.devices;

            for (did, kind) in devs {
                if matches!(kind, DeviceKind::Output) {
                    self.end = self
                        .graph
                        .dev_ins
                        .get(did)
                        .and_then(|ins| ins.first())
                        .copied();
                }
            }
            // TODO: Handle insertion of multiple outputs

            self.qt.insert(a, id);

            None
        } else {
            Some(id)
        }
    }

    pub fn show(&mut self, ctx: &Context, ui: &mut Ui) -> Option<StackResponse> {
        let rect = ui.available_rect_before_wrap();
        let top = rect.left_top();
        let rects: SecondaryMap<_, _> = self
            .qt
            .iter()
            .map(|e| {
                let a = e.area();
                let Point { x, y } = a.anchor();
                let tl = top + vec2(x as f32 * U1_WIDTH, y as f32 * U1_HEIGHT);
                let sz = vec2(a.width() as f32 * U1_WIDTH, a.height() as f32 * U1_HEIGHT);
                (*e.value_ref(), Rect::from_min_size(tl, sz))
            })
            .collect();

        let mut trigger_rebuild = false;
        let mut control_change = None;
        for (mid, r) in &rects {
            ui.put(*r, |ui: &mut Ui| {
                let InnerResponse { inner, response } = self.graph[mid].show(ui);

                match (self.attempting_connection, inner) {
                    (_, ModuleResponse::None) => {}
                    (ConnAttempt::None, ModuleResponse::AttemptConnection(Connector::Out(id))) => {
                        self.attempting_connection = ConnAttempt::Out(id);
                    }
                    (ConnAttempt::None, ModuleResponse::AttemptConnection(Connector::In(id))) => {
                        self.attempting_connection = ConnAttempt::In(id);
                    }
                    (ConnAttempt::In(_), ModuleResponse::AttemptConnection(Connector::In(_))) => {}
                    (
                        ConnAttempt::In(inid),
                        ModuleResponse::AttemptConnection(Connector::Out(outid)),
                    ) => {
                        self.graph.cables.insert(inid, outid);
                        self.attempting_connection = ConnAttempt::None;
                        trigger_rebuild = true;
                    }
                    (
                        ConnAttempt::Out(outid),
                        ModuleResponse::AttemptConnection(Connector::In(inid)),
                    ) => {
                        self.graph.cables.insert(inid, outid);
                        self.attempting_connection = ConnAttempt::None;
                        trigger_rebuild = true;
                    }
                    (ConnAttempt::Out(_), ModuleResponse::AttemptConnection(Connector::Out(_))) => {
                    }
                    (_, ModuleResponse::Changed(conn, v)) => control_change = Some((conn, v)),
                    (
                        ConnAttempt::In(_) | ConnAttempt::Out(_),
                        ModuleResponse::AttemptDisconnect(_),
                    ) => self.attempting_connection = ConnAttempt::None,
                    (_, ModuleResponse::AttemptDisconnect(c)) => {
                        match c {
                            Connector::In(i) => {
                                self.graph.cables.remove(i);
                            }
                            Connector::Out(o) => self.graph.cables.retain(|_, co| *co != o),
                        };
                        trigger_rebuild = true;
                    }
                }

                response
            });
            let p = ui.painter();
            p.rect_stroke(
                *r,
                Rounding::ZERO,
                Stroke {
                    width: 2.,
                    color: Color32::from_rgb(80, 140, 0),
                },
            );
        }

        self.draw_wires(rects, ctx, ui);

        if trigger_rebuild {
            self.end
                .as_ref()
                .map(|end| StackResponse::Rebuild(self.graph.walk_to(*end)))
        } else if let Some((c, v)) = control_change {
            Some(StackResponse::ControlChange(c, v))
        } else {
            None
        }
    }

    fn draw_wires(&mut self, rects: SecondaryMap<ModuleId, Rect>, ctx: &Context, ui: &mut Ui) {
        match self.attempting_connection {
            ConnAttempt::None => {}
            ConnAttempt::Out(start) => {
                let start = self.get_output_pos(start, &rects);
                if let Some(end) = ctx.pointer_latest_pos() {
                    draw_catenary(start, end, ui.painter());
                }
            }
            ConnAttempt::In(start) => {
                let start = self.get_input_pos(start, &rects);
                if let Some(end) = ctx.pointer_latest_pos() {
                    draw_catenary(start, end, ui.painter());
                }
            }
        }

        for (inp, &out) in &self.graph.cables {
            let start = self.get_input_pos(inp, &rects);

            let end = self.get_output_pos(out, &rects);

            draw_catenary(start, end, ui.painter());
        }
    }

    fn get_output_pos(&self, out: OutputId, rects: &SecondaryMap<ModuleId, Rect>) -> Pos2 {
        let (mid, module, vid) = self
            .graph
            .modules
            .iter()
            .find_map(|(mid, m)| m.outs.get(out).map(|o| (mid, m, *o)))
            .unwrap();
        let mod_tl = rects[mid].min;
        let widget = &*module.visuals[vid];

        mod_tl + (widget.pos().to_vec2() + widget.size() / 2.0)
    }

    fn get_input_pos(&self, inp: InputId, rects: &SecondaryMap<ModuleId, Rect>) -> Pos2 {
        let (mid, module, vid) = self
            .graph
            .modules
            .iter()
            .find_map(|(mid, m)| m.ins.get(inp).map(|o| (mid, m, *o)))
            .unwrap();
        let mod_tl = rects[mid].min;
        let widget = &*module.visuals[vid];

        mod_tl + (widget.pos().to_vec2() + widget.size() / 2.0)
    }
}

pub enum StackResponse {
    Rebuild(CtlGraph),
    ControlChange(Connector, f32),
    MidiChange(STQueue<(u64, MidiMessage<'static>)>),
}

impl std::fmt::Debug for StackResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rebuild(arg0) => f.debug_tuple("Rebuild").field(arg0).finish(),
            Self::ControlChange(arg0, arg1) => f
                .debug_tuple("ControlChange")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::MidiChange(_) => f.debug_tuple("MidiChange").finish(),
        }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}
