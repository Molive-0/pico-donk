// constants
use crate::libm::{exp2, sin};
use fixed::types::I16F16;
use fixed::types::I8F24;

pub type Sample = I8F24;
pub type Half = I16F16;

#[macro_export]
macro_rules! s { ($($a:tt)+) => { fixed_macro::fixed!($($a)+: I8F24) } }
#[macro_export]
macro_rules! h { ($($a:tt)+) => { fixed_macro::fixed!($($a)+: I16F16) } }

const fn sample_from_num(x: f64) -> Sample {
    Sample::from_bits((x * exp2(Sample::FRAC_NBITS as f64)) as i32)
}
const fn half_from_num(x: f64) -> Half {
    Half::from_bits((x * exp2(Half::FRAC_NBITS as f64)) as i32)
}

pub const FAST_SIN_TAB_LOG2_SIZE: usize = 9; // size = 512
pub const FAST_SIN_TAB_SIZE: usize = 1 << FAST_SIN_TAB_LOG2_SIZE;
pub const ADJUSTED_FAST_SIN_TAB_SIZE: usize = FAST_SIN_TAB_SIZE + 1;
pub const FAST_SIN_TAB: [Sample; ADJUSTED_FAST_SIN_TAB_SIZE] = {
    let mut array = [Sample::from_bits(0); ADJUSTED_FAST_SIN_TAB_SIZE];
    let mut f = 0;
    while f < ADJUSTED_FAST_SIN_TAB_SIZE {
        array[f] = sample_from_num(
            ((sin((f as f64) * (core::f64::consts::TAU / FAST_SIN_TAB_SIZE as f64)) + 1.) / 2.)
                + 1.,
        );
        f += 1;
    }
    array
};

pub const NOTE_TAB_SIZE: usize = 128;
pub const NOTE_TAB: [Half; NOTE_TAB_SIZE] = {
    let mut array = [Half::from_bits(0); NOTE_TAB_SIZE];
    let mut f = 0;
    while f < NOTE_TAB_SIZE {
        array[f] = half_from_num(exp2((f as f64 - 69.0) / 12.0) * 440.0);
        f += 1;
    }
    array
};

pub const FAST_EXP_TAB_LOG2_SIZE: usize = 9; // size = 512
pub const FAST_EXP_TAB_SIZE: usize = 1 << FAST_EXP_TAB_LOG2_SIZE;
pub const ADJUSTED_FAST_EXP_TAB_SIZE: usize = FAST_EXP_TAB_SIZE + 1;
pub const FAST_EXP_TAB: [Sample; ADJUSTED_FAST_EXP_TAB_SIZE] = {
    let mut array = [Sample::from_bits(0); ADJUSTED_FAST_EXP_TAB_SIZE];
    let mut f = 0;
    while f < ADJUSTED_FAST_EXP_TAB_SIZE {
        array[f] = sample_from_num(exp2(f as f64 / FAST_EXP_TAB_SIZE as f64) - 1.);
        f += 1;
    }
    array
};

pub const DRUMS: &'static [u8] = include_bytes!("../dat/cw_amen08_165.raw");
pub const LENGTH: usize = DRUMS.len() / 2;
