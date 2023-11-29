use egui::{
    vec2,
    Vec2,
};
use serde::{
    Deserialize,
    Serialize,
};

pub const U1_HEIGHT: f32 = 128.0;
pub const U1_WIDTH: f32 = U1_HEIGHT;

pub const H1_HEIGHT: f32 = U1_HEIGHT;
pub const H1_WIDTH: f32 = U1_WIDTH;

pub const Q1_HEIGHT: f32 = U1_HEIGHT * 2.0;
pub const Q1_WIDTH: f32 = U1_WIDTH * 2.0;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ModuleSize {
    U1,
    U2,
    U3,
    U4,

    H1,
    H2,
    H3,
    H4,

    Q1,
    Q2,
}

impl std::fmt::Display for ModuleSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl ModuleSize {
    pub fn all() -> [Self; 10] {
        use ModuleSize::*;
        [U1, U2, U3, U4, H1, H2, H3, H4, Q1, Q2]
    }

    pub fn size(&self) -> Vec2 {
        match self {
            Self::U1 => vec2(U1_WIDTH * 1.0, U1_HEIGHT),
            Self::U2 => vec2(U1_WIDTH * 2.0, U1_HEIGHT),
            Self::U3 => vec2(U1_WIDTH * 3.0, U1_HEIGHT),
            Self::U4 => vec2(U1_WIDTH * 4.0, U1_HEIGHT),
            Self::H1 => vec2(H1_WIDTH, H1_HEIGHT * 1.0),
            Self::H2 => vec2(H1_WIDTH, H1_HEIGHT * 2.0),
            Self::H3 => vec2(H1_WIDTH, H1_HEIGHT * 3.0),
            Self::H4 => vec2(H1_WIDTH, H1_HEIGHT * 4.0),
            Self::Q1 => vec2(Q1_WIDTH, Q1_HEIGHT),
            Self::Q2 => vec2(Q1_WIDTH * 2.0, Q1_HEIGHT * 2.0),
        }
    }

    pub fn size_in_units(&self) -> (u8, u8) {
        match self {
            Self::U1 => (1, 1),
            Self::U2 => (2, 1),
            Self::U3 => (3, 1),
            Self::U4 => (4, 1),
            Self::H1 => (1, 1),
            Self::H2 => (1, 2),
            Self::H3 => (1, 3),
            Self::H4 => (1, 4),
            Self::Q1 => (2, 2),
            Self::Q2 => (4, 4),
        }
    }
}
