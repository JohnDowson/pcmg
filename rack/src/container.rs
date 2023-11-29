use std::collections::BTreeMap;

use eframe::{
    egui::{
        Response,
        Sense,
        Ui,
    },
    epaint::{
        vec2,
        Color32,
        Rect,
    },
};
use egui::Pos2;
use itertools::Itertools;
use quadtree_rs::{
    area::AreaBuilder,
    point::Point,
    Quadtree,
};
use serde::{
    Deserialize,
    Serialize,
};
use slotmap::SecondaryMap;

use crate::{
    devices::{
        DeviceDescription,
        DEVICES,
    },
    graph::{
        Graph,
        InputId,
        ModuleId,
        OutputId,
    },
    widget_description::{
        ModuleDescription,
        WidgetDescription,
        WidgetKind,
    },
    widgets::{
        connector::Cable,
        SlotWidget,
    },
};

use self::sizing::*;

pub mod module;
pub mod sizing;

pub struct Stack {
    pub graph: Graph,
    wires: Vec<Cable>,
    qt: Quadtree<u8, ModuleId>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            graph: Default::default(),
            wires: Default::default(),
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

    pub fn show(&mut self, ui: &mut Ui) {
        let rect = ui.available_rect_before_wrap();
        let top = rect.left_top();
        let rects = self.qt.iter().map(|e| {
            let a = e.area();
            let Point { x, y } = a.anchor();
            let tl = top + vec2(x as f32 * U1_WIDTH, y as f32 * U1_HEIGHT);
            let sz = vec2(a.width() as f32 * U1_WIDTH, a.height() as f32 * U1_HEIGHT);
            (*e.value_ref(), Rect::from_min_size(tl, sz))
        });

        for (im, r) in rects {
            let m = &mut self.graph[im];
            ui.put(r, |ui: &mut Ui| m.show(ui));
            let p = ui.painter();
            p.debug_rect(r, Color32::from_rgb(0, 255, 0), format!("{im:?}"));
        }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}
