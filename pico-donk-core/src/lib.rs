#![no_std]
//we about to be doing some cursed const
#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![feature(const_fn_trait_bound)]

#[macro_use]
pub mod cst;
#[macro_use]
pub mod helpers;
pub mod device;
pub mod synth;

#[macro_export]
macro_rules! note {
    ($x:expr) => {
        ($x / 48000.0 * 65536.0) as u16
    };
}

pub struct Song {
    current_tempo: u32,
    i: usize,
    leads: [u16; 4],
    bass: u16,
    first: bool,
    drums: &'static [u8],
}

impl Song {
    pub fn new() -> Song {
        Song {
            current_tempo: 120,
            i: 0,
            leads: [0; 4],
            bass: 0,
            first: true,
            drums: sampler_cache!(cst::DRUMS),
        }
    }

    pub fn get_sample(&mut self) -> u16 {
        let mut output: u16 = 0;
        output = output.saturating_add(self.get_drums(self.i, cst::LENGTH) / 2);
        if self.first {
            output = output.saturating_add(self.get_bass(self.i, cst::LENGTH) / 3);
            output = output.saturating_add(self.get_lead(self.i, cst::LENGTH) / 3);
        } else {
            output = output.saturating_add(self.get_second_lead(self.i, cst::LENGTH) / 3);
        }
        self.i += 1;
        if self.i >= cst::LENGTH {
            self.i = 0;
            self.bass = 0;
            self.leads = [0; 4];
            self.first = !self.first;
        }
        output
    }

    fn get_drums(&mut self, i: usize, length: usize) -> u16 {
        let mut drums_vol = ((cst::DRUMS[(i * 4) % length] as u16)
            + ((self.drums[(i * 4 + 1) % length] as u16) << 8))
            .wrapping_add(0x8000);
        if drums_vol > 50000 {
            drums_vol = ((drums_vol - 50000) / 32) + 50000;
        }
        drums_vol
    }

    fn get_bass(&mut self, i: usize, length: usize) -> u16 {
        const NOTES: [u16; 8] = [
            note!(130.81), //C3
            note!(130.81), //C3
            note!(155.56), //Eb3
            note!(155.56), //Eb3
            note!(174.61), //F3
            note!(174.61), //F3
            note!(207.65), //Ab3
            note!(196.00), //G3
        ];
        let note = NOTES[i * 8 / length];
        self.bass = self.bass.wrapping_add(note * 2);
        if self.bass > 32767 {
            return (65535 - (self.bass as i32)) as u16 * 2;
        } else {
            return self.bass * 2;
        }
    }

    fn get_lead(&mut self, i: usize, length: usize) -> u16 {
        const NOTES: [u16; 32] = [
            note!(130.81), //C3
            note!(130.81),
            note!(130.81),
            note!(130.81),
            note!(130.81),
            note!(123.47), //B2
            note!(130.81), //C3
            note!(155.56), //Eb3
            note!(155.56), //Eb3
            note!(155.56), //Eb3
            note!(155.56), //Eb3
            note!(155.56), //Eb3
            note!(155.56), //Eb3
            note!(155.56), //Eb3
            note!(155.56), //Eb3
            note!(155.56), //Eb3
            note!(174.61), //F3
            note!(174.61), //F3
            note!(174.61), //F3
            note!(174.61), //F3
            note!(174.61), //F3
            note!(155.56), //Eb3
            note!(174.61), //F3
            note!(207.65), //Ab3
            note!(207.65), //Ab3
            note!(207.65), //Ab3
            note!(207.65), //Ab3
            note!(207.65), //Ab3
            note!(196.00), //G3
            note!(196.00), //G3
            note!(196.00), //G3
            note!(196.00), //G3
        ];
        let note = NOTES[i * 32 / length];
        let mut offset = -2;
        let mut output = 0;
        for lead in self.leads.iter_mut() {
            *lead = lead.wrapping_add(((note * 4) as i32 + offset) as u16);
            output += *lead / 4;
            offset += 1;
        }
        output
    }

    fn get_second_lead(&mut self, i: usize, length: usize) -> u16 {
        const NOTES: [u16; 32] = [
            note!(0.0),
            note!(0.0),
            note!(196.00),
            note!(0.0),
            note!(0.0),
            note!(196.00),
            note!(0.0),
            note!(0.0),
            note!(0.0),
            note!(0.0),
            note!(196.00),
            note!(0.0),
            note!(196.00),
            note!(174.61),
            note!(196.00),
            note!(0.0),
            note!(0.0),
            note!(0.0),
            note!(196.00),
            note!(0.0),
            note!(0.0),
            note!(196.00),
            note!(0.0),
            note!(0.0),
            note!(0.0),
            note!(0.0),
            note!(196.00),
            note!(0.0),
            note!(196.00),
            note!(207.65),
            note!(196.00),
            note!(0.0),
        ];
        let note = NOTES[i * 32 / length];
        let mut offset = -6;
        let mut output = 0;
        for lead in self.leads.iter_mut() {
            *lead = lead.wrapping_add(((note * 4) as i32 + offset) as u16);
            output += *lead / 4;
            offset += 3;
        }
        output
    }
}
