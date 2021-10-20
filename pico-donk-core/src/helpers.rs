use fixed::types::U16F16;

use crate::cst::*;

static mut RANDOM_SEED: i32 = 1;

#[inline]
pub fn rand_float() -> Sample {
    unsafe {
        RANDOM_SEED = RANDOM_SEED.wrapping_mul(0x15a4e35);
        Sample::from_bits(RANDOM_SEED & 0x3FFF << 16 | RANDOM_SEED & -65536 >> 16)
    }
}

#[inline]
//TODO: optimise this
pub fn exp_2(x: U16F16) -> U16F16 {
    U16F16::from_num(2f32.powf(x.to_num()))
}

pub trait SinCos {
    //Takes a number between 1 and 2 and returns a number between 0 and 3
    fn cos(self) -> Sample;
    fn sin(self) -> Sample;
}

impl SinCos for Sample {
    #[inline]
    fn cos(self) -> Sample {
        (self + FP5).sin()
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
