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
