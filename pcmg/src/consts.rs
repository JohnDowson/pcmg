pub const SAMPLERATE: f64 = 48000.;
pub const BPM: f64 = 120.;
#[allow(non_snake_case)]
pub mod F64 {
    use std::f64::consts::PI;
    pub const A4: f64 = 440.; //E 329.63;
    pub const TWO_PI: f64 = 2. * PI;
    pub const TWELFTH_ROOT_OF_2: f64 = 1.059_463_094_359_295_3;
}
#[allow(non_snake_case)]
pub mod F32 {
    use std::f32::consts::PI;
    pub const A4: f32 = 440.; //E 329.63;
    pub const TWO_PI: f32 = 2. * PI;
    pub const TWELFTH_ROOT_OF_2: f32 = 1.059_463_1;
}
