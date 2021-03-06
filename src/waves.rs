use std::f64::consts::PI;
#[allow(dead_code)]
pub fn sine(freq: f64, t: f64) -> f64 {
    (freq * t).sin()
}
#[allow(dead_code)]
pub fn snare(freq: f64, t: f64) -> f64 {
    (freq * t).powf(2.).sin() / 1.5
}
#[allow(dead_code)]
pub fn squeaky(freq: f64, t: f64) -> f64 {
    (freq * t).tan().sin()
}
#[allow(dead_code)]
pub fn square(freq: f64, t: f64) -> f64 {
    (freq * t).sin().signum()
}
#[allow(dead_code)]
pub fn triangle(freq: f64, t: f64) -> f64 {
    (freq * t).sin().asin()
}
#[allow(dead_code)]
pub fn sawtooth(freq: f64, t: f64) -> f64 {
    (freq * t).tan().atan()
}
#[allow(dead_code)]
pub fn phat_sine(freq: f64, t: f64) -> f64 {
    sine(freq, t) + sine(freq / PI, t)
}
#[allow(dead_code)]
pub fn noize(freq: f64, t: f64) -> f64 {
    use byteorder::{ByteOrder, LittleEndian};
    use rand::prelude::*;
    let mut b = [0u8; 32];
    LittleEndian::write_f64(&mut b, freq * t);
    let mut rng: StdRng = SeedableRng::from_seed(b);
    rng.gen_range::<f64, f64, f64>(-1., 1.)
}
