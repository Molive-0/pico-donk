use crate::cst::*;
use crate::helpers::*;

#[derive(Debug, Clone, Copy, Default)]
enum EnvelopeState {
    Attack,
    Decay,
    Sustain,
    Release,
    #[default]
    Finished,
}

#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    state: EnvelopeState,

    attack: Sample,
    decay: Sample,
    sustain: Sample,
    release: Sample,

    pos: Sample,
    release_value: Sample,
}

impl Default for Envelope {
    fn default() -> Self {
        Envelope::new()
    }
}

impl Envelope {
    fn new() -> Envelope {
        Envelope {
            state: EnvelopeState::Finished,
            attack: s!(1),
            decay: s!(5),
            sustain: sf!(0.5),
            release: sf!(1.5),
            pos: s!(0),
            release_value: sf!(1.5),
        }
    }
    fn trigger(&mut self) {
        self.state = EnvelopeState::Attack;
        self.pos = s!(0);
    }
    fn off(&mut self) {
        self.release_value = self.getValue();
        self.state = EnvelopeState::Release;
        self.pos = s!(0);
    }
    fn getValue(&self) -> Sample {
        match self.state {
            EnvelopeState::Attack => self.pos * self.attack,

            EnvelopeState::Decay => {
                let f = s!(1) - (self.pos * self.decay);
                let ff = f * f;
                (ff) + (self.sustain * (s!(1) - ff))
            }

            EnvelopeState::Sustain => self.sustain,

            EnvelopeState::Release => {
                let f = s!(1) - (self.pos * self.release);
                let ff = f * f;
                self.release_value * ff
            }

            _ => {
                s!(0)
            }
        }
    }
    fn next(&mut self) {}

    fn set_attack(&mut self, value: Sample) {
        self.attack = s!(1) / value;
    }
    fn set_decay(&mut self, value: Sample) {
        self.decay = s!(1) / value;
    }
    fn set_sustain(&mut self, value: Sample) {
        self.sustain = value;
    }
    fn set_release(&mut self, value: Sample) {
        self.release = s!(1) / value;
    }
    fn get_attack(&mut self) -> Sample {
        s!(1) / self.attack
    }
    fn get_decay(&mut self) -> Sample {
        s!(1) / self.decay
    }
    fn get_sustain(&mut self) -> Sample {
        self.sustain
    }
    fn get_release(&mut self) -> Sample {
        s!(1) / self.release
    }
}
