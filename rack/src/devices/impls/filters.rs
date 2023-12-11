pub struct MoogFilter {
    input: f32,

    cutoff: f32,
    sample_rate: f32,
    resonance: f32,

    y1: f32,
    y2: f32,
    y3: f32,
    y4: f32,
    oldx: f32,
    oldy1: f32,
    oldy2: f32,
    oldy3: f32,
    x: f32,
    r: f32,
    p: f32,
    k: f32,
}

impl MoogFilter {
    pub fn new(sample_rate: f32, cutoff: f32, resonance: f32) -> Self {
        let mut res = Self {
            input: 0.0,
            sample_rate,
            cutoff,
            resonance,
            y1: 0.0,
            y2: 0.0,
            y3: 0.0,
            y4: 0.0,
            oldx: 0.0,
            oldy1: 0.0,
            oldy2: 0.0,
            oldy3: 0.0,
            x: 0.0,
            r: 0.0,
            p: 0.0,
            k: 0.0,
        };
        res.calculate();
        res
    }

    fn calculate(&mut self) {
        let f = (self.cutoff + self.cutoff) / self.sample_rate;
        let p = f * (1.8 - 0.8 * f);
        let k = p + p - 1.0;

        let t = (1.0 - p) * 1.386249;
        let t2 = 12.0 + t * t;
        let r = self.resonance * (t2 + 6.0 * t) / (t2 - 6.0 * t);
        self.r = r;
        self.p = p;
        self.k = k;
    }

    pub fn set_input(&mut self, input: f32) {
        self.input = input;
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
        self.calculate();
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
        self.calculate();
    }

    pub fn filter(&mut self) -> f32 {
        // process input
        self.x = self.input - self.r * self.y4;

        //Four cascaded onepole filters (bilinear transform)
        self.y1 = self.x * self.p + self.oldx * self.p - self.k * self.y1;
        self.y2 = self.y1 * self.p + self.oldy1 * self.p - self.k * self.y2;
        self.y3 = self.y2 * self.p + self.oldy2 * self.p - self.k * self.y3;
        self.y4 = self.y3 * self.p + self.oldy3 * self.p - self.k * self.y4;

        //Clipper band limited sigmoid
        self.y4 -= (self.y4 * self.y4 * self.y4) / 6.0;

        self.oldx = self.x;
        self.oldy1 = self.y1;
        self.oldy2 = self.y2;
        self.oldy3 = self.y3;
        self.y4
    }
}
