use std::{
    collections::BTreeMap,
    default,
};

use egui::Color32;
use emath::{
    Pos2,
    Vec2,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::widget_description::WidgetKind;

#[derive(Serialize, Deserialize, Default)]
pub struct WidgetTemplate {
    pub kind: WidgetKind,
    positon: Pos2,
    pub size: Vec2,
    // Needs to be a btreemap to keep track of components across insertions
    // only matters in the editor, can be serialized as vec
    #[serde(serialize_with = "crate::ser_btree_as_vec")]
    #[serde(deserialize_with = "crate::de_vec_as_btree")]
    pub components: BTreeMap<usize, VisualComponent>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct VisualComponent {
    pub shape: Vec<Pos2>,
    pub color: VisualColor,
    pub show: Activity,
    pub thickness: f32,
}

enum VisualShape {}

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

#[derive(Serialize, Deserialize)]
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
