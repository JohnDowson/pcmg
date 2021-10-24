// TODO
pub struct ADSR<T: num::Float> {
    attack: T,
    decay: T,
    sustain: T,
    release: T,
}
impl<T: num::Float> ADSR<T> {
    pub fn new(attack: T, decay: T, sustain: T, release: T) -> ADSR<T> {
        ADSR {
            attack,
            decay,
            sustain,
            release,
        }
    }
    pub fn apply(&self, amplitude: T, time: T) -> T {
        let alpha = if time <= self.attack {
            //attack
            time / self.attack
        } else if time >= (self.attack + self.decay + self.sustain) {
            //release
            ((self.attack + self.decay + self.sustain) - time) / self.decay
        } else {
            //sustain
            T::from(1.).unwrap()
        };
        amplitude * alpha
    }
}
