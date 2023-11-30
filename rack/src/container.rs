use eframe::{
    egui::Ui,
    epaint::{
        vec2,
        Color32,
        Rect,
    },
};

use egui::{
    epaint::PathShape,
    Context,
    InnerResponse,
    Rounding,
    Stroke,
};
use itertools::Itertools;
use quadtree_rs::{
    area::AreaBuilder,
    point::Point,
    Quadtree,
};
use slotmap::SecondaryMap;

use crate::{
    graph::{
        Graph,
        ModuleId,
    },
    widgets::connector::{
        catenary,
        InAddr,
        OutAddr,
    },
};

use self::{
    module::ModuleResponse,
    sizing::*,
};

pub mod module;
pub mod sizing;

pub struct Stack {
    pub graph: Graph,
    // wires: Vec<Cable>,
    attempting_connection: ConnAttempt,
    qt: Quadtree<u8, ModuleId>,
}

#[derive(Clone, Copy)]
enum ConnAttempt {
    None,
    In(InAddr),
    Out(OutAddr),
}

impl Stack {
    pub fn new() -> Self {
        Self {
            graph: Default::default(),
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
            self.qt.insert(a, id);
            None
        } else {
            Some(id)
        }
    }

    pub fn show(&mut self, ctx: &Context, ui: &mut Ui) {
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

        for (im, r) in &rects {
            ui.put(*r, |ui: &mut Ui| {
                let InnerResponse { inner, response } = self.graph[im].show(ui);

                match (self.attempting_connection, inner) {
                    (_, ModuleResponse::None) => {}
                    (ConnAttempt::None, ModuleResponse::AttemptConnectionOut(id)) => {
                        self.attempting_connection = ConnAttempt::Out(OutAddr { mid: im, wid: id });
                    }
                    (ConnAttempt::None, ModuleResponse::AttemptConnectionIn(id)) => {
                        self.attempting_connection = ConnAttempt::In(InAddr { mid: im, wid: id });
                    }
                    (ConnAttempt::In(_), ModuleResponse::AttemptConnectionIn(_)) => {}
                    (ConnAttempt::In(inid), ModuleResponse::AttemptConnectionOut(outid)) => {
                        let outid = OutAddr {
                            mid: im,
                            wid: outid,
                        };
                        self.graph.cables.insert(inid.wid.0, outid.wid.0);
                        self.attempting_connection = ConnAttempt::None;
                    }
                    (ConnAttempt::Out(outid), ModuleResponse::AttemptConnectionIn(inid)) => {
                        let inid = InAddr { mid: im, wid: inid };
                        self.graph.cables.insert(inid.wid.0, outid.wid.0);
                        self.attempting_connection = ConnAttempt::None;
                    }
                    (ConnAttempt::Out(_), ModuleResponse::AttemptConnectionOut(_)) => {}
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
    }

    fn draw_wires(&mut self, rects: SecondaryMap<ModuleId, Rect>, ctx: &Context, ui: &mut Ui) {
        match self.attempting_connection {
            ConnAttempt::None => {}
            ConnAttempt::Out(start) => {
                let start = self.get_output_pos(start.wid.0, &rects);
                if let Some(end) = ctx.pointer_latest_pos() {
                    draw_catenary(start, end, ui)
                }
            }
            ConnAttempt::In(start) => {
                let start = self.get_input_pos(start.wid.0, &rects);
                if let Some(end) = ctx.pointer_latest_pos() {
                    draw_catenary(start, end, ui)
                }
            }
        }

        for (inp, &out) in &self.graph.cables {
            let start = self.get_input_pos(inp, &rects);

            let end = self.get_output_pos(out, &rects);

            draw_catenary(start, end, ui);
        }
    }

    fn get_output_pos(
        &self,
        out: crate::graph::OutputId,
        rects: &SecondaryMap<ModuleId, Rect>,
    ) -> emath::Pos2 {
        let mid = self.graph.outs[out];
        let module = &self.graph.modules[mid];
        let mod_tl = rects[mid].min;
        let wid = module.out_ass[out];
        let widget = &module.contents[wid];

        mod_tl + (widget.pos().to_vec2() + widget.size() / 2.0)
    }

    fn get_input_pos(
        &self,
        inp: crate::graph::InputId,
        rects: &SecondaryMap<ModuleId, Rect>,
    ) -> emath::Pos2 {
        let mid = self.graph.ins[inp];
        let module = &self.graph.modules[mid];
        let mod_tl = rects[mid].min;
        let wid = module.in_ass[inp];
        let widget = &module.contents[wid];

        mod_tl + (widget.pos().to_vec2() + widget.size() / 2.0)
    }
}

fn draw_catenary(start: emath::Pos2, end: emath::Pos2, ui: &mut Ui) {
    let pts = catenary(start, end, 0.6, 0.10, 16).collect();
    ui.painter().add(PathShape::line(
        pts,
        Stroke {
            width: 2.0,
            color: Color32::RED,
        },
    ));
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}
