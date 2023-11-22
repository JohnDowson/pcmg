use std::f32::consts::{PI, TAU};

pub struct KrajeskiLadder {
    cutoff: f32,
    resonance: f32,
    sample_rate: f32,
    state: [f32; 5],
    delay: [f32; 5],
    wc: f32,
    g: f32,
    g_res: f32,
    g_comp: f32,
    drive: f32,
}

impl KrajeskiLadder {
    pub fn new(sample_rate: f32, cutoff: f32, resonance: f32) -> Self {
        let mut this = Self {
            cutoff: Default::default(),
            resonance: Default::default(),
            sample_rate,
            state: Default::default(),
            delay: Default::default(),
            wc: Default::default(),
            g: Default::default(),
            g_res: Default::default(),
            g_comp: 1.0,
            drive: 1.0,
        };

        this.set_cutoff(cutoff);
        this.set_resonance(resonance);

        this
    }

    pub fn filter(&mut self, sample: f32) -> f32 {
        self.state[0] = (self.drive
            * (sample - 4.0 * self.g_res * (self.state[4] - self.g_comp * sample)))
            .tanh();

        for i in 0..self.state.len() - 1 {
            self.state[i + 1] = self.g
                * (0.3 / 1.3 * self.state[i] + 1.0 / 1.3 * self.delay[i] - self.state[i + 1])
                + self.state[i + 1];
            self.delay[i] = self.state[i];
        }
        self.state[4]
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
        self.wc = TAU * cutoff / self.sample_rate;
        self.g = 0.9892 * self.wc - std::f32::consts::LOG10_E * self.wc.powi(2)
            + 0.1381 * self.wc.powi(3)
            - 0.0202 * self.wc.powi(4);
    }
    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
        self.g_res = resonance
            * (1.0029 + 0.0526 * self.wc - 0.926 * self.wc.powi(2) + 0.0218 * self.wc.powi(3));
    }
}

pub struct MoogFilter {
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

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
        self.calculate();
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
        self.calculate();
    }

    pub fn filter(&mut self, signal: f32) -> f32 {
        // process input
        self.x = signal - self.r * self.y4;

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

pub struct ResonantIIRLowpass {
    sample_rate: f32,
    res_freq: f32,
    amp: f32,
    w: f32,
    q: f32,
    r: f32,
    c: f32,
    vibra_pos: f32,
    vibra_speed: f32,
}

impl ResonantIIRLowpass {
    pub fn new(sample_rate: f32, res_freq: f32, amp: f32) -> Self {
        let w = 2.0 * PI * res_freq / sample_rate;
        let q = 1.0 - w / (2.0 * (amp + 0.5 / (1.0 + w)) + w - 2.0);
        let r = q * q;
        let c = r + 1.0 - 2.0 * w.cos() * q;

        Self {
            sample_rate,
            res_freq,
            amp,
            w,
            q,
            r,
            c,
            vibra_pos: 0.0,
            vibra_speed: 0.0,
        }
    }

    pub fn filter(&mut self, signal: f32) -> f32 {
        self.vibra_speed += (signal - self.vibra_speed) * self.c;

        /* Add velocity to vibra's position */
        self.vibra_pos += self.vibra_speed;

        /* Attenuate/amplify vibra's velocity by resonance */
        self.vibra_speed *= self.r;
        self.vibra_pos
    }
}

pub struct FourPoleFilter {
    coefs: [f32; 9],
    d: [f32; 4],
    w: f32,
    g: f32,
}

impl FourPoleFilter {
    pub fn new_lp(peak_freq: f32, peak_mag: f32) -> Self {
        let mut coefs = [0.0; 9];
        let w = peak_freq;
        let g = peak_mag;
        let k = (4.0 * g - 3.0) / (g + 1.0);
        let mut p = 1.0 - 0.25 * k;
        p *= p;

        let a = 1.0 / ((0.5 * w).tan() * (1.0 + p));
        p = 1.0 + a;
        let q = 1.0 - a;

        let a0 = 1.0 / (k + p * p * p * p);
        let a1 = 4.0 * (k + p * p * p * q);
        let a2 = 6.0 * (k + p * p * q * q);
        let a3 = 4.0 * (k + p * q * q * q);
        let a4 = k + q * q * q * q;
        p = a0 * (k + 1.0);

        coefs[0] = p;
        coefs[1] = 4.0 * p;
        coefs[2] = 6.0 * p;
        coefs[3] = 4.0 * p;
        coefs[4] = p;
        coefs[5] = -a1 * a0;
        coefs[6] = -a2 * a0;
        coefs[7] = -a3 * a0;
        coefs[8] = -a4 * a0;

        Self {
            coefs,
            d: [0.0; 4],
            w,
            g,
        }
    }

    pub fn new_hp(peak_freq: f32, peak_mag: f32) -> Self {
        let mut coefs = [0.0; 9];
        let w = peak_freq;
        let g = peak_mag;
        let k = (4.0 * g - 3.0) / (g + 1.0);
        let mut p = 1.0 - 0.25 * k;
        p *= p;

        let a = (0.5 * w).tan() / (1.0 + p);
        p = a + 1.0;
        let q = a - 1.0;

        let a0 = 1.0 / (p * p * p * p + k);
        let a1 = 4.0 * (p * p * p * q - k);
        let a2 = 6.0 * (p * p * q * q + k);
        let a3 = 4.0 * (p * q * q * q - k);
        let a4 = q * q * q * q + k;
        p = a0 * (k + 1.0);

        coefs[0] = p;
        coefs[1] = -4.0 * p;
        coefs[2] = 6.0 * p;
        coefs[3] = -4.0 * p;
        coefs[4] = p;
        coefs[5] = -a1 * a0;
        coefs[6] = -a2 * a0;
        coefs[7] = -a3 * a0;
        coefs[8] = -a4 * a0;

        Self {
            coefs,
            d: [0.0; 4],
            w,
            g,
        }
    }

    pub fn filter(&mut self, signal: f32) -> f32 {
        let out = self.coefs[0] * signal + self.d[0];

        self.d[0] = self.coefs[1] * signal + self.coefs[5] * out + self.d[1];
        self.d[1] = self.coefs[2] * signal + self.coefs[6] * out + self.d[2];
        self.d[2] = self.coefs[3] * signal + self.coefs[7] * out + self.d[3];
        self.d[3] = self.coefs[4] * signal + self.coefs[8] * out;

        out
    }
}
