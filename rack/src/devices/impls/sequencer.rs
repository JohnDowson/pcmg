pub struct Sequencer {
    samplerate: f32,
    samples_to_next: f32,
    samples_since_last: f32,
    duration: f32,
    next: usize,
    pub spb: f32,
    pub sequence: [f32; 8],
}

impl Sequencer {
    pub fn new(samplerate: f32) -> Self {
        Self {
            samplerate,
            samples_to_next: 0.,
            samples_since_last: 0.,
            duration: 200.,
            next: 0,
            spb: 0.,
            sequence: [0.; 8],
        }
    }

    pub fn set_spb(&mut self, spb: f32) {
        self.spb = spb;
        self.samples_to_next = self.samplerate * spb;
    }

    pub fn get_output(&mut self) -> f32 {
        if self.samples_to_next < 1. {
            self.next += 1;
            self.next %= 8;
            self.samples_to_next = self.samplerate * self.spb;
            self.samples_since_last = 0.;
        }

        if self.samples_since_last <= self.duration {
            let sample = self.sequence[self.next];
            self.samples_since_last += 1.0;
            self.samples_to_next -= 1.0;
            sample
        } else {
            self.samples_to_next -= 1.0;
            0.0
        }
    }
}
