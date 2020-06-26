use std::f64::consts::PI;
#[allow(dead_code)]
pub fn sine(x: f64) -> f64 {
    x.sin()
}
#[allow(dead_code)]
pub fn snare(x: f64) -> f64 {
    x.powf(2.).sin() / 1.5
}
#[allow(dead_code)]
pub fn squeaky(x: f64) -> f64 {
    x.tan().sin()
}
#[allow(dead_code)]
pub fn square(x: f64) -> f64 {
    x.sin().signum()
}
#[allow(dead_code)]
pub fn triangle(x: f64) -> f64 {
    x.sin().asin()
}
#[allow(dead_code)]
pub fn sawtooth(x: f64) -> f64 {
    x.tan().atan()
}
#[allow(dead_code)]
pub fn phat_sine(x: f64) -> f64 {
    sine(x) + sine(x / PI)
}
#[allow(dead_code)]
pub fn noize(x: f64) -> f64 {
    use rand::prelude::*;
    use byteorder::{LittleEndian, ByteOrder};
    let mut b = [0u8; 32];
    LittleEndian::write_f64(&mut b, x);
    let mut rng: StdRng = SeedableRng::from_seed(b);
    rng.gen_range::<f64, f64, f64>(x-2., x)
}