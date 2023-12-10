use egui::{
    epaint::{
        CircleShape,
        PathShape,
        RectShape,
        TextShape,
    },
    Color32,
    FontFamily,
    FontId,
    Rounding,
    Shape,
    Stroke,
    Ui,
};
use emath::{
    vec2,
    Pos2,
    Rect,
};
use serde::{
    Deserialize,
    Serialize,
};

use self::templates::{
    VisualComponentTemplate,
    VisualShapeTemplate,
};

pub mod templates;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Static,
    Rotate,
    ShiftX,
    ShiftY,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl Mode {
    pub fn all() -> [Self; 4] {
        use Mode::*;
        [Static, Rotate, ShiftX, ShiftY]
    }
}

#[derive(Serialize, Deserialize)]
pub struct VisualComponent {
    pub shape: VisualShape,
    pub color: VisualColor,
    pub show: Activity,
    pub mode: Mode,
    pub thickness: f32,
}

impl VisualComponent {
    pub fn show(&self, ui: &mut Ui, widget_center: Pos2, theme: VisualTheme) {
        self.show_with_value(ui, widget_center, theme, 0.0)
    }

    pub fn show_with_value(
        &self,
        ui: &mut Ui,
        widget_center: Pos2,
        theme: VisualTheme,
        value: f32,
    ) {
        let color = match self.color {
            VisualColor::Highlight => theme.highlight_color,
            VisualColor::Midtone => theme.midtone_color,
            VisualColor::Lowlight => theme.lowlight_color,
            VisualColor::Accent => theme.accent_color,
            VisualColor::Text => theme.text_color,
        };

        let translation = {
            match self.mode {
                Mode::Static => {
                    Box::new(|pos: Pos2| widget_center + pos.to_vec2()) as Box<dyn Fn(_) -> _>
                }
                Mode::Rotate => Box::new(|pos: Pos2| {
                    let Pos2 { x, y } = pos;
                    let nx = x * value.cos() - y * value.sin();
                    let ny = y * value.cos() + x * value.sin();
                    widget_center + vec2(nx, ny)
                }),
                Mode::ShiftX => {
                    Box::new(|pos: Pos2| widget_center + pos.to_vec2() * vec2(value, 0.0))
                }
                Mode::ShiftY => {
                    Box::new(|pos: Pos2| widget_center + pos.to_vec2() * vec2(0.0, value))
                }
            }
        };

        let shape: Shape = match self.shape.clone() {
            VisualShape::Line(mut line) => {
                line.iter_mut().for_each(|p| *p = translation(*p));
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
                translation(p),
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

                TextShape::new(translation(p) - galley.size() / 2.0, galley).into()
            }
            VisualShape::Rect(min, max, fc) => {
                let fill = match fc {
                    Some(VisualColor::Highlight) => theme.highlight_color,
                    Some(VisualColor::Midtone) => theme.midtone_color,
                    Some(VisualColor::Lowlight) => theme.lowlight_color,
                    Some(VisualColor::Accent) => theme.accent_color,
                    Some(VisualColor::Text) => theme.text_color,
                    None => Color32::TRANSPARENT,
                };
                RectShape::new(
                    Rect::from_min_max(translation(min), translation(max)),
                    Rounding::ZERO,
                    fill,
                    Stroke {
                        width: self.thickness,
                        color,
                    },
                )
                .into()
            }
        };
        ui.painter().add(shape);
    }
}

impl TryFrom<VisualComponentTemplate> for VisualComponent {
    type Error = ();

    fn try_from(value: VisualComponentTemplate) -> Result<Self, Self::Error> {
        let VisualComponentTemplate {
            shape,
            color,
            show,
            thickness,
            mode,
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
            VisualShapeTemplate::Rect(min, max, fill) => {
                VisualShape::Rect(min.ok_or(())?, max.ok_or(())?, fill)
            }
        };
        Ok(VisualComponent {
            shape,
            color,
            show,
            thickness,
            mode,
        })
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

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum VisualShape {
    Line(Vec<Pos2>),
    Rect(Pos2, Pos2, Option<VisualColor>),
    Circle(Pos2, f32),
    Text(Pos2, String, FontFamily),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct VisualTheme {
    pub highlight_color: Color32,
    pub midtone_color: Color32,
    pub lowlight_color: Color32,
    pub accent_color: Color32,
    pub text_color: Color32,
    pub background_color: Color32,
    pub background_accent_color: Color32,
}

impl Default for VisualTheme {
    fn default() -> Self {
        Self {
            highlight_color: Color32::WHITE,
            midtone_color: Color32::GRAY,
            lowlight_color: Color32::DARK_GRAY,
            accent_color: Color32::GOLD,
            text_color: Color32::GRAY,
            background_color: Color32::from_rgb(40, 80, 40),
            background_accent_color: Color32::from_rgb(60, 100, 40),
        }
    }
}

// module or entire rack has a set theme
#[derive(Serialize, Deserialize)]
struct ThemeId(usize);

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
