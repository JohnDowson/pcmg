use std::collections::BTreeMap;

use egui::{
    FontFamily,
    Ui,
};
use emath::{
    Pos2,
    Vec2,
};
use serde::{
    Deserialize,
    Serialize,
};
use uuid::Uuid;

use crate::{
    widget_description::WidgetKind,
    widgets::{
        connector::ports::Port,
        knob::Knob,
        SlotWidget,
    },
    Uuidentified,
};

use super::{
    Activity,
    Mode,
    VisualColor,
    VisualComponent,
    VisualTheme,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct WidgetTemplate {
    pub uuid: Uuid,
    pub name: String,
    pub kind: WidgetKind,
    pub position: Pos2,
    pub size: Vec2,
    // Needs to be a btreemap to keep track of components across insertions
    // only matters in the editor, can be serialized as vec
    #[serde(serialize_with = "crate::ser_btree_as_vec")]
    #[serde(deserialize_with = "crate::de_vec_as_btree")]
    pub components: BTreeMap<usize, VisualComponentTemplate>,
}

impl Default for WidgetTemplate {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: Default::default(),
            kind: Default::default(),
            position: Default::default(),
            size: Default::default(),
            components: Default::default(),
        }
    }
}

impl Uuidentified for WidgetTemplate {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

impl std::fmt::Debug for WidgetTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WidgetTemplate")
            .field("uuid", &self.uuid)
            .field("kind", &self.kind)
            .field("position", &self.position)
            .field("size", &self.size)
            .finish()
    }
}

impl WidgetTemplate {
    pub fn preview(&self, ui: &mut Ui, pos: Pos2, theme: VisualTheme, _value: f32) {
        for component in self.components.values() {
            if let Ok(c) = component.clone().try_into() {
                VisualComponent::show(&c, ui, pos, theme)
            }
        }
    }

    pub fn into_slot_widget(self) -> Option<SlotWidget> {
        match self.kind {
            WidgetKind::Knob(_) => Knob::from_template(self).map(SlotWidget::Knob),
            WidgetKind::Port => Port::from_template(self).map(SlotWidget::Port),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VisualComponentTemplate {
    pub shape: VisualShapeTemplate,
    pub color: VisualColor,
    pub show: Activity,
    pub mode: Mode,
    pub thickness: f32,
}

impl Default for VisualComponentTemplate {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            color: Default::default(),
            show: Default::default(),
            thickness: 1.0,
            mode: Mode::Static,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum VisualShapeTemplate {
    Line(Vec<Pos2>),
    Circle(Option<Pos2>, Option<f32>),
    Text(Option<Pos2>, String, FontFamily),
}

impl std::fmt::Debug for VisualShapeTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Line(..) => write!(f, "Line"),
            Self::Circle(..) => write!(f, "Circle"),
            VisualShapeTemplate::Text(..) => write!(f, "Text"),
        }
    }
}

impl std::fmt::Display for VisualShapeTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl VisualShapeTemplate {
    pub fn pop(&mut self) {
        match self {
            VisualShapeTemplate::Line(shape) => {
                shape.pop();
            }
            VisualShapeTemplate::Circle(pos, rad) => {
                rad.take().map(|_| ()).or_else(|| pos.take().map(|_| ()));
            }
            VisualShapeTemplate::Text(pos, _, _) => {
                pos.take();
            }
        }
    }

    pub fn push(&mut self, pos: Pos2) {
        match self {
            VisualShapeTemplate::Line(shape) => shape.push(pos),
            VisualShapeTemplate::Circle(p, r) => {
                if let Some(p) = p {
                    *r = Some((*p - pos).length())
                } else {
                    *p = Some(pos);
                }
            }
            VisualShapeTemplate::Text(p, _, _) => *p = Some(pos),
        }
    }
}

impl Default for VisualShapeTemplate {
    fn default() -> Self {
        Self::Circle(None, None)
    }
}
