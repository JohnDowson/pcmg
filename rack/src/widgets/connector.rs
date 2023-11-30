use emath::Pos2;

use crate::graph::{
    InputId,
    ModuleId,
    OutputId,
};

pub mod ports;

pub trait Addr {
    type Wid;
    fn mid(&self) -> ModuleId;
    fn wid(&self) -> (Self::Wid, u16);
}

#[derive(Clone, Copy)]
pub struct OutAddr {
    pub mid: ModuleId,
    pub wid: (OutputId, u16),
}

impl Addr for OutAddr {
    type Wid = OutputId;

    fn mid(&self) -> ModuleId {
        self.mid
    }

    fn wid(&self) -> (Self::Wid, u16) {
        self.wid
    }
}

#[derive(Clone, Copy)]
pub struct InAddr {
    pub mid: ModuleId,
    pub wid: (InputId, u16),
}

impl Addr for InAddr {
    type Wid = InputId;

    fn mid(&self) -> ModuleId {
        self.mid
    }

    fn wid(&self) -> (Self::Wid, u16) {
        self.wid
    }
}

pub fn catenary(start: Pos2, end: Pos2, h: f32, m: f32, n: usize) -> impl Iterator<Item = Pos2> {
    fn find_t0(k: f32, c: f32) -> f32 {
        if c == 0.0 {
            return 0.5;
        }

        let a = k.cosh();
        let b = k.sinh();

        let d = 1.0 - (a - b);

        let r = (c * c + b * b - a * a + a + a - 1.0).sqrt();

        ((r - c) / d).ln() / k
    }

    let w = (end.x - start.x).abs().sqrt();
    let a = (h * w.ln()) / (m * w.ln());
    let a = -a; // invert gravity because in egui +y is down
    let k = w / a.abs();
    let c = (end.y - start.y) / a;
    let t0 = find_t0(k, c);
    let y0 = start.y - a * (-w * t0 / a).cosh();

    (0..=n).map(move |i| {
        let t = i as f32 / n as f32;
        let x = (1.0 - t) * start.x + t * end.x;
        let y = y0 + a * (w * (t - t0) / a).cosh();
        Pos2 { x, y }
    })
}
