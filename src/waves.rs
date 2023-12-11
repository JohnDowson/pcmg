use num_traits::Float;

pub fn snare<T: Float>(freq: T) -> T {
    freq.powf(T::from(2.).unwrap()).sin() / T::from(1.5).unwrap()
}

pub fn squeaky<T: Float>(freq: T) -> T {
    (freq).tan().sin()
}

pub fn square<T: Float>(freq: T) -> T {
    freq.sin().signum()
}

pub fn triangle<T: Float>(freq: T) -> T {
    freq.sin().asin()
}

pub fn sawtooth<T: Float>(freq: T) -> T {
    freq.tan().atan()
}

pub fn phat_sine(freq: f32) -> f32 {
    freq.sin() + (freq / std::f32::consts::PI).sin()
}
