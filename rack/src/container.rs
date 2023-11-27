use std::collections::BTreeMap;

use eframe::{
    egui::{Response, Sense, Ui},
    epaint::{vec2, Color32, Rect},
};
use egui::Pos2;
use itertools::Itertools;
use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
use serde::{Deserialize, Serialize};

use crate::{
    widget_description::{ModuleDescription, Sid, WidgetDescription},
    widgets::SlotWidget,
};

use self::sizing::*;

pub mod sizing;

pub struct Stack {
    modules: Vec<Box<dyn Module>>,
    qt: Quadtree<u8, usize>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            qt: Quadtree::new(2),
        }
    }

    pub fn with_module<M: Module + 'static>(&mut self, module: M) -> Option<M> {
        let sz = module.module_size();
        let i = self.modules.len();

        let mut ab = AreaBuilder::default();
        ab.dimensions(sz.size_in_units());

        let a = (0..self.qt.width())
            .cartesian_product(0..self.qt.height())
            .map(|(x, y)| {
                let (x, y) = (x as _, y as _);
                ab.anchor(Point { x, y });
                ab.build().unwrap()
            })
            // .find(|c| !self.qt.regions().any(|a| a.intersects(*c)));
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
            self.qt.insert(a, i);
            self.modules.push(Box::new(module));
            None
        } else {
            Some(module)
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let rect = ui.available_rect_before_wrap();
        // let p = ui.painter();
        // p.debug_rect(rect, Color32::from_rgb(0, 180, 180), "");
        let top = rect.left_top();
        let rects = self.qt.iter().map(|e| {
            let a = e.area();
            let Point { x, y } = a.anchor();
            let tl = top + vec2(x as f32 * U1_WIDTH, y as f32 * U1_HEIGHT);
            let sz = vec2(a.width() as f32 * U1_WIDTH, a.height() as f32 * U1_HEIGHT);
            (*e.value_ref(), Rect::from_min_size(tl, sz))
        });

        for (im, r) in rects {
            let m = &mut self.modules[im];
            ui.put(r, |ui: &mut Ui| m.show(ui));
            let p = ui.painter();
            p.debug_rect(r, Color32::from_rgb(0, 255, 0), format!("{im}"));
        }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Module {
    fn module_size(&self) -> SlotSize;

    fn show(&mut self, ui: &mut Ui) -> Response;
}

pub struct Slot {
    size: SlotSize,
    contents: Vec<Box<dyn SlotWidget>>,
    state: SlotState,
}

pub type SlotState = BTreeMap<usize, SlotWidgetState>;
pub type SlotWidgetState = BTreeMap<&'static str, StateValue>;

impl Slot {
    pub fn empty(size: SlotSize) -> Self {
        Self {
            size,
            contents: Default::default(),
            state: Default::default(),
        }
    }

    pub fn new(sid: Sid, size: SlotSize, contents: Vec<WidgetDescription>) -> Self {
        let contents = contents.into_iter().map(|w| w.dyn_widget(sid)).collect();
        Self {
            size,
            contents,
            state: Default::default(),
        }
    }

    pub fn from_description(sid: Sid, description: ModuleDescription) -> Self {
        let ModuleDescription { size, widgets } = description;
        Self::new(sid, size, widgets)
    }
}

impl Slot {
    fn ui_for(&mut self, position: Pos2, ui: &mut Ui) {
        let mut contents = std::mem::take(&mut self.contents);
        for (i, w) in contents.iter_mut().enumerate() {
            let pos = w.pos() + position.to_vec2();
            self.state.entry(i).or_default();
            ui.put(Rect::from_min_size(pos, w.size()), |ui: &mut Ui| {
                w.ui(ui, &mut self.state)
            });
        }
        self.contents = contents;
    }
}

impl Module for Slot {
    fn module_size(&self) -> SlotSize {
        self.size
    }

    fn show(&mut self, ui: &mut Ui) -> Response {
        let size = self.module_size().size();

        let resp = ui.allocate_response(self.module_size().size(), Sense::click_and_drag());

        self.ui_for(resp.rect.min, ui);
        let p = ui.painter();
        p.debug_rect(
            Rect::from_center_size(resp.rect.center(), size),
            Color32::from_rgb(255, 0, 0),
            "",
        );
        p.debug_rect(resp.rect, Color32::from_rgb(0, 0, 255), "");

        resp
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StateValue {
    Float(f32),
    Bool(bool),
    Range(f32, f32),
}

impl std::fmt::Display for StateValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateValue::Float(v) => v.fmt(f),
            StateValue::Bool(b) => b.fmt(f),
            StateValue::Range(s, e) => write!(f, "{s}..{e}"),
        }
    }
}

impl std::str::FromStr for StateValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s {
            "true" => Self::Bool(true),
            "false" => Self::Bool(false),
            s => {
                let res = s
                    .split("..")
                    .map(|fv| fv.parse())
                    .collect::<Result<Vec<f32>, _>>()
                    .map_err(|_| ())?;
                match res.len() {
                    1 => Self::Float(res[0]),
                    2 => Self::Range(res[0], res[1]),
                    _ => return Err(()),
                }
            }
        };
        Ok(v)
    }
}