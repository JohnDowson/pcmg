pub struct Attenuator {
    input: f32,
    factor: f32,
}

impl Attenuator {
    pub fn new() -> Self {
        Self {
            input: 0.0,
            factor: 1.0,
        }
    }

    pub fn set_input(&mut self, input: f32) {
        self.input = input;
    }

    pub fn set_factor(&mut self, factor: f32) {
        self.factor = factor;
    }

    pub fn get_output(&self) -> f32 {
        self.input * self.factor
    }
}

impl Default for Attenuator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AbMixer {
    a: f32,
    b: f32,
    ratio: f32,
}

impl AbMixer {
    pub fn new() -> Self {
        Self {
            a: 0.0,
            b: 0.0,
            ratio: 0.5,
        }
    }

    pub fn set_a(&mut self, a: f32) {
        self.a = a;
    }

    pub fn set_b(&mut self, b: f32) {
        self.b = b;
    }

    pub fn get_output(&self) -> f32 {
        self.a * (1.0 - self.ratio) + self.b * self.ratio
    }
}

impl Default for AbMixer {
    fn default() -> Self {
        Self::new()
    }
}
