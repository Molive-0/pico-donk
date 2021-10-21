use core::ops::{Deref, DerefMut};

use crate::cst::*;

static mut RANDOM_SEED: i32 = 1;

#[inline]
pub fn rand_float() -> Sample {
    unsafe {
        RANDOM_SEED = RANDOM_SEED.wrapping_mul(0x15a4e35);
        Sample::from_bits(RANDOM_SEED & 0x3FFF << 16 | RANDOM_SEED & -65536 >> 16)
    }
}

pub trait Exp2 {
    fn exp_2(self) -> Self;
}

impl Exp2 for Sample {
    #[inline]
    //TODO: optimise this
    fn exp_2(self) -> Self {
        assert!(self >= 0);
        Self::from_num(2f32.powf(self.to_num()))
    }
}

impl Exp2 for Half {
    #[inline]
    //TODO: optimise this
    fn exp_2(self) -> Self {
        assert!(self >= 0);
        Self::from_num(2f32.powf(self.to_num()))
    }
}

pub trait SinCos {
    //Takes a number between 1 and 2 and returns a number between 0 and 3
    fn cos(self) -> Self;
    fn sin(self) -> Self;
}

impl SinCos for Sample {
    #[inline]
    fn cos(self) -> Sample {
        self.wrapping_add(FP25).sin()
    }
    fn sin(self) -> Sample {
        const FRACT_BITS: i32 = Sample::FRAC_NBITS as i32 - FAST_SIN_TAB_LOG2_SIZE as i32;
        const FRACT_SCALE: i32 = 1 << FRACT_BITS;
        const FRACT_MASK: i32 = FRACT_SCALE - 1;

        let significand = self.frac().to_bits();
        let index = (significand >> FRACT_BITS) as usize;
        let fract_mix = significand & FRACT_MASK;

        let left = FAST_SIN_TAB[index];
        let right = FAST_SIN_TAB[index + 1];

        let offset = right - left;
        let offset = ((offset.to_bits() >> 15) * (fract_mix >> (15 - FAST_SIN_TAB_LOG2_SIZE))) << 1;

        left + Sample::from_bits(offset)
    }
}

pub trait Squares {
    fn square_135(self) -> Sample;
    fn square_35(self) -> Sample;
}

impl Squares for Sample {
    #[inline]
    fn square_135(self) -> Sample {
        self.sin() + ((self * 3).sin() / 3) + ((self * 5).sin() / 5)
    }
    #[inline]
    fn square_35(self) -> Sample {
        ((self * 3).sin() / 3) + ((self * 5).sin() / 5)
    }
}

pub trait Mix {
    fn mix(self, other: Self, mix: Self) -> Self;
}

impl Mix for Sample {
    fn mix(self, other: Self, mix: Self) -> Self {
        (self * (FP1 - mix)) + (other * mix)
    }
}

impl Mix for Half {
    fn mix(self, other: Self, mix: Self) -> Self {
        (self * (Half::from_num(1) - mix)) + (other * mix)
    }
}

macro_rules! structs {
    ($type: ident, $name: ident) => {
        pub struct $name {
            value: $type,
        }
        impl Deref for $name {
            type Target = $type;
            fn deref(&self) -> &Self::Target {
                &self.value
            }
        }
        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.value
            }
        }
    };
}

structs!(Half, Note);
pub struct Freq {
    value: Half,
}
pub struct Db {
    value: Half,
}
pub struct EnvValue {
    value: Sample,
}
pub struct Volume {
    value: Sample,
}
pub struct Param {
    value: Sample,
}
pub struct HalfParam {
    value: Half,
}
pub struct Q {
    value: Sample,
}
pub struct Resonance {
    value: Sample,
}
pub struct Unisolo {
    value: Half,
}
pub struct VibratoFreq {
    value: Half,
}
pub struct Pan {
    value: Sample,
}
pub struct Spread {
    value: Sample,
}
pub struct VoiceMode {
    value: Sample,
}

impl From<Note> for Freq {
    fn from(n: Note) -> Self {
        Freq::from_num(n)
    }
}
