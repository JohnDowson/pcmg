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
        Cable,
        Waddr,
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
    wires: Vec<Cable>,
    attempting_connection: ConnAttempt,
    qt: Quadtree<u8, ModuleId>,
}

#[derive(Clone, Copy)]
enum ConnAttempt {
    None,
    In(Waddr),
    Out(Waddr),
}

impl Stack {
    pub fn new() -> Self {
        Self {
            graph: Default::default(),
            wires: Default::default(),
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
            let m = &mut self.graph[im];
            ui.put(*r, |ui: &mut Ui| {
                let InnerResponse { inner, response } = m.show(ui);

                match (self.attempting_connection, inner) {
                    (_, ModuleResponse::None) => {}
                    (ConnAttempt::None, ModuleResponse::AttemptConnectionOut(id)) => {
                        self.attempting_connection = ConnAttempt::Out(Waddr { mid: im, wid: id });
                    }
                    (ConnAttempt::None, ModuleResponse::AttemptConnectionIn(id)) => {
                        self.attempting_connection = ConnAttempt::In(Waddr { mid: im, wid: id });
                    }
                    (ConnAttempt::In(_), ModuleResponse::AttemptConnectionIn(_)) => {}
                    (ConnAttempt::In(inid), ModuleResponse::AttemptConnectionOut(outid)) => {
                        let outid = Waddr {
                            mid: im,
                            wid: outid,
                        };
                        self.wires.push(Cable {
                            a_id: inid,
                            b_id: outid,
                        });
                        self.attempting_connection = ConnAttempt::None;
                    }
                    (ConnAttempt::Out(outid), ModuleResponse::AttemptConnectionIn(inid)) => {
                        let inid = Waddr { mid: im, wid: inid };
                        self.wires.push(Cable {
                            a_id: outid,
                            b_id: inid,
                        });
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

            match self.attempting_connection {
                ConnAttempt::None => {}
                ConnAttempt::In(start) | ConnAttempt::Out(start) => {
                    let start_mod = &self.graph.modules[start.mid];
                    let mod_tl = rects[start.mid].min;
                    let start_wid = &start_mod.contents[start.wid as usize];
                    let start = mod_tl + (start_wid.pos().to_vec2() + start_wid.size() / 2.0);
                    if let Some(end) = ctx.pointer_latest_pos() {
                        let pts = catenary(start, end, 0.4, 0.15, 16).collect();

                        ui.painter().add(PathShape::line(
                            pts,
                            Stroke {
                                width: 2.0,
                                color: Color32::RED,
                            },
                        ));
                    }
                }
            }

            for Cable { a_id, b_id } in &self.wires {
                let module = &self.graph.modules[a_id.mid];
                let mod_tl = rects[a_id.mid].min;
                let widget = &module.contents[a_id.wid as usize];
                let start = mod_tl + (widget.pos().to_vec2() + widget.size() / 2.0);

                let module = &self.graph.modules[b_id.mid];
                let mod_tl = rects[b_id.mid].min;
                let widget = &module.contents[b_id.wid as usize];
                let end = mod_tl + (widget.pos().to_vec2() + widget.size() / 2.0);
                let pts = catenary(start, end, 0.4, 0.15, 16).collect();
                ui.painter().add(PathShape::line(
                    pts,
                    Stroke {
                        width: 2.0,
                        color: Color32::RED,
                    },
                ));
            }
        }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}
