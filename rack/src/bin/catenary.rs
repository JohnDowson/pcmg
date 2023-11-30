use egui::{
    epaint::PathShape,
    CentralPanel,
    Color32,
    Context,
    DragValue,
    Sense,
    Slider,
    Stroke,
    TopBottomPanel,
};
use emath::{
    vec2,
    Rect,
    Vec2,
};
use rack::widgets::connector::catenary;

pub struct Catenary {
    start: Vec2,
    end: Vec2,
    h: f32,
    m: f32,
    n: usize,
}

impl Default for Catenary {
    fn default() -> Self {
        Self {
            start: vec2(0., 0.),
            end: vec2(10., 10.),
            h: 1.0,
            m: 1.0,
            n: 16,
        }
    }
}

impl eframe::App for Catenary {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("h");
                ui.add(Slider::new(&mut self.h, 0.0..=2.0));
            });
            ui.horizontal(|ui| {
                ui.label("m");
                ui.add(Slider::new(&mut self.m, 0.0..=2.0));
            });
            ui.horizontal(|ui| {
                ui.label("n");
                ui.add(DragValue::new(&mut self.n));
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            let sz = ui.available_size();
            let (r, _) = ui.allocate_exact_size(sz, Sense::hover());
            let center = r.center();
            let (start, end) = (center + self.start, center + self.end);

            let ar = ui.allocate_rect(Rect::from_center_size(start, vec2(4., 4.)), Sense::drag());
            let br = ui.allocate_rect(Rect::from_center_size(end, vec2(4., 4.)), Sense::drag());
            let p = ui.painter();
            p.debug_rect(ar.rect, Color32::from_rgb(255, 0, 0), "a");
            p.debug_rect(br.rect, Color32::from_rgb(0, 255, 0), "b");

            let pts = catenary(start, end, self.h, self.m, self.n).collect();
            p.add(PathShape::line(
                pts,
                Stroke::new(2.0, Color32::from_rgb(0, 140, 0)),
            ));

            self.start += ar.drag_delta();
            self.end += br.drag_delta();
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Catenary::default();

    eframe::run_native(
        "catenary",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app)),
    )?;
    Ok(())
}
