use egui::{
    epaint::PathShape,
    Color32,
    Painter,
    Stroke,
};
use emath::Pos2;

pub mod ports;

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

pub fn draw_catenary(start: emath::Pos2, end: emath::Pos2, painter: &Painter) {
    let pts: Vec<_> = catenary(start, end, 0.6, 0.10, 16).collect();
    painter.add(PathShape::line(
        pts.clone(),
        Stroke {
            width: 4.0,
            color: Color32::RED,
        },
    ));
    painter.add(PathShape::line(
        pts,
        Stroke {
            width: 3.0,
            color: Color32::DARK_RED,
        },
    ));
    painter.circle_filled(start, 6.0, Color32::DARK_GRAY);
    painter.circle_filled(end, 6.0, Color32::DARK_GRAY);
}
