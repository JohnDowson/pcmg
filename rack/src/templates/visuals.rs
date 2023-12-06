use std::collections::BTreeMap;

use egui::{
    epaint::{
        CircleShape,
        PathShape,
        TextShape,
    },
    Color32,
    FontFamily,
    FontId,
    Shape,
    Stroke,
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

use crate::widget_description::WidgetKind;

#[derive(Serialize, Deserialize, Default)]
pub struct WidgetTemplate {
    pub uuid: Uuid,
    pub kind: WidgetKind,
    positon: Pos2,
    pub size: Vec2,
    // Needs to be a btreemap to keep track of components across insertions
    // only matters in the editor, can be serialized as vec
    #[serde(serialize_with = "crate::ser_btree_as_vec")]
    #[serde(deserialize_with = "crate::de_vec_as_btree")]
    pub components: BTreeMap<usize, VisualComponentTemplate>,
}

impl WidgetTemplate {
    pub fn preview(&self, ui: &mut Ui, pos: Pos2, theme: VisualTheme, value: f32) {
        for component in self.components.values() {
            if let Ok(c) = component.clone().try_into() {
                VisualComponent::show(&c, ui, pos, theme, value)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VisualComponent {
    pub shape: VisualShape,
    pub color: VisualColor,
    pub show: Activity,
    pub thickness: f32,
}

impl VisualComponent {
    pub fn show(&self, ui: &mut Ui, pos: Pos2, theme: VisualTheme, _value: f32) {
        let color = match self.color {
            VisualColor::Highlight => theme.highlight_color,
            VisualColor::Midtone => theme.midtone_color,
            VisualColor::Lowlight => theme.lowlight_color,
            VisualColor::Accent => theme.accent_color,
            VisualColor::Text => theme.text_color,
        };
        let shape: Shape = match self.shape.clone() {
            VisualShape::Line(mut line) => {
                line.iter_mut().for_each(|p| *p = pos + p.to_vec2());
                if line.first() == line.last() {
                    line.pop();
                    PathShape::closed_line(
                        line,
                        Stroke {
                            width: self.thickness,
                            color,
                        },
                    )
                    .into()
                } else {
                    PathShape::line(
                        line,
                        Stroke {
                            width: self.thickness,
                            color,
                        },
                    )
                    .into()
                }
            }
            VisualShape::Circle(p, r) => CircleShape::stroke(
                pos + p.to_vec2(),
                r,
                Stroke {
                    width: self.thickness,
                    color,
                },
            )
            .into(),
            VisualShape::Text(p, t, f) => {
                let galley = ui.fonts(|r| {
                    r.layout_no_wrap(
                        t,
                        FontId {
                            size: self.thickness,
                            family: f,
                        },
                        color,
                    )
                });

                TextShape::new((pos + p.to_vec2()) - galley.size() / 2.0, galley).into()
            }
        };
        ui.painter().add(shape);
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VisualComponentTemplate {
    pub shape: VisualShapeTemplate,
    pub color: VisualColor,
    pub show: Activity,
    pub thickness: f32,
}

impl TryFrom<VisualComponentTemplate> for VisualComponent {
    type Error = ();

    fn try_from(value: VisualComponentTemplate) -> Result<Self, Self::Error> {
        let VisualComponentTemplate {
            shape,
            color,
            show,
            thickness,
        } = value;
        let shape = match shape {
            VisualShapeTemplate::Line(l) => {
                if l.is_empty() {
                    return Err(());
                }
                VisualShape::Line(l)
            }
            VisualShapeTemplate::Circle(p, r) => VisualShape::Circle(p.ok_or(())?, r.ok_or(())?),
            VisualShapeTemplate::Text(p, t, f) => VisualShape::Text(p.ok_or(())?, t, f),
        };
        Ok(VisualComponent {
            shape,
            color,
            show,
            thickness,
        })
    }
}

impl Default for VisualComponentTemplate {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            color: Default::default(),
            show: Default::default(),
            thickness: 1.0,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum VisualShape {
    Line(Vec<Pos2>),
    Circle(Pos2, f32),
    Text(Pos2, String, FontFamily),
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

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone, Copy)]
pub enum Activity {
    #[default]
    Always,
    OnHover,
    OnInteract,
}

impl Activity {
    pub fn all() -> [Self; 3] {
        use Activity::*;
        [Always, OnHover, OnInteract]
    }
}

impl std::fmt::Display for Activity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default)]
pub enum VisualColor {
    Highlight,
    #[default]
    Midtone,
    Lowlight,
    Accent,
    Text,
}

impl std::fmt::Display for VisualColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl VisualColor {
    pub fn all() -> [Self; 5] {
        use VisualColor::*;
        [Highlight, Midtone, Lowlight, Accent, Text]
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct VisualTheme {
    highlight_color: Color32,
    midtone_color: Color32,
    lowlight_color: Color32,
    accent_color: Color32,
    text_color: Color32,
}

// module or entire rack has a set theme
#[derive(Serialize, Deserialize)]
struct ThemeId(usize);
