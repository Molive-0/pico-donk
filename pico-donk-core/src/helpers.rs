use core::ops::{Deref, DerefMut};

use fixed::{types::extra::LeEqU32, FixedI32};
use fixed_sqrt::FixedSqrt;

use crate::cst::*;

static mut RANDOM_SEED: i32 = 1;

pub trait Rand {
    fn rand() -> Self;
}

impl<T: LeEqU32> Rand for FixedI32<T> {
    #[inline]
    fn rand() -> Self {
        unsafe {
            RANDOM_SEED = RANDOM_SEED.wrapping_mul(0x15a4e35);
            Self::from_bits(RANDOM_SEED & 0x3FFF << 16 | RANDOM_SEED & -65536 >> 16)
        }
    }
}

pub trait TableLookup
where
    Self: Sized,
{
    fn lookup(self, table: &[Self], table_log_size: usize) -> Self;
}

impl<T: LeEqU32> TableLookup for FixedI32<T> {
    fn lookup(self, table: &[Self], table_log_size: usize) -> Self {
        let fract_bits: i32 = Self::FRAC_NBITS as i32 - table_log_size as i32;
        let fract_scale: i32 = 1 << fract_bits;
        let fract_mask: i32 = fract_scale - 1;

        let significand = self.frac().to_bits();
        let index = (significand >> fract_bits) as usize;
        let fract_mix = significand & fract_mask;

        let left = table[index];
        let right = table[index + 1];

        let offset = right - left;
        let offset = ((offset.to_bits() >> (16 - Self::INT_NBITS / 2))
            * (fract_mix >> ((16 - Self::INT_NBITS / 2) - table_log_size as u32)))
            << (Self::INT_NBITS % 2);
        left + Self::from_bits(offset)
    }
}

pub trait Exp2 {
    fn exp2(self) -> Self;
}

impl Exp2 for Sample {
    fn exp2(self) -> Self {
        let scale: Self = if self > 0 {
            Self::from_bits(s!(1).to_bits() << self.floor().to_num::<i32>())
        } else {
            Self::from_bits(s!(1).to_bits() >> self.floor().abs().to_num::<i32>())
        };

        scale + (self.lookup(&FAST_EXP_TAB, FAST_EXP_TAB_LOG2_SIZE) * scale)
    }
}

impl Exp2 for Half {
    fn exp2(self) -> Self {
        let shift = self.floor().to_num::<i32>();
        let scale: Self = if self > 0 {
            Self::from_bits(h!(1).to_bits() << shift)
        } else {
            Self::from_bits(h!(1).to_bits() >> shift.abs())
        };
        let offset = (Sample::wrapping_from_num(self)
            .lookup(&FAST_EXP_TAB, FAST_EXP_TAB_LOG2_SIZE)
            .to_bits()
            >> (Sample::FRAC_NBITS as i32 - Half::FRAC_NBITS as i32 - shift).clamp(0, 31))
            << (shift - (Sample::FRAC_NBITS as i32 - Half::FRAC_NBITS as i32)).clamp(0, 31);

        scale + Half::from_bits(offset)
    }
}

pub trait SinCos {
    //Takes a number between 1 and 2 and returns a number between 0 and 3
    fn cos(self) -> Self;
    fn sin(self) -> Self;
}

impl SinCos for Sample {
    #[inline]
    fn cos(self) -> Self {
        self.wrapping_add(s!(0.25)).sin()
    }
    #[inline]
    fn sin(self) -> Self {
        self.lookup(&FAST_SIN_TAB, FAST_SIN_TAB_LOG2_SIZE)
    }
}

impl SinCos for Half {
    #[inline]
    fn cos(self) -> Self {
        self.wrapping_add(h!(0.25)).sin()
    }
    #[inline]
    fn sin(self) -> Self {
        Self::from_num(
            Sample::wrapping_from_num(self).lookup(&FAST_SIN_TAB, FAST_SIN_TAB_LOG2_SIZE),
        )
    }
}

pub trait Squares {
    fn square_135(self) -> Self;
    fn square_35(self) -> Self;
}

impl Squares for Sample {
    #[inline]
    fn square_135(self) -> Self {
        self.sin() + ((self * s!(3)).sin() * s!(0.33333333333)) + ((self * s!(5)).sin() * s!(0.2))
    }
    #[inline]
    fn square_35(self) -> Self {
        ((self * s!(3)).sin() * s!(0.33333333333)) + ((self * s!(5)).sin() * s!(0.2))
    }
}

impl Squares for Half {
    #[inline]
    fn square_135(self) -> Self {
        self.sin() + ((self * h!(3)).sin() * h!(0.33333333333)) + ((self * h!(5)).sin() * h!(0.2))
    }
    #[inline]
    fn square_35(self) -> Self {
        ((self * h!(3)).sin() * h!(0.33333333333)) + ((self * h!(5)).sin() * h!(0.2))
    }
}

macro_rules! structs {
    ($name: ident, $type: ty) => {
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
        impl From<i32> for $name {
            fn from(t: i32) -> Self {
                <$type>::from_num(t).into()
            }
        }
        impl From<$name> for i32 {
            fn from(t: $name) -> Self {
                let t: $type = t.into();
                t.to_num()
            }
        }
    };
}
macro_rules! self_convert {
    ($name: ident, $type: ty) => {
        impl From<$type> for $name {
            fn from(t: $type) -> Self {
                $name { value: t }
            }
        }
        impl From<$name> for $type {
            fn from(t: $name) -> Self {
                t.value
            }
        }
    };
}

structs!(Note, Half);
structs!(Freq, Half);
structs!(Db, Half);
structs!(EnvValue, Sample);
structs!(Volume, Sample);
structs!(Param, Sample);
structs!(HalfParam, Half);
structs!(Q, Sample);
structs!(Resonance, Sample);
structs!(Unisolo, Half);
structs!(VibratoFreq, Half);
structs!(Pan, Sample);
structs!(Spread, Sample);
structs!(VoiceMode, Sample);
self_convert!(Note, Half);
self_convert!(Freq, Half);
self_convert!(Param, Sample);
self_convert!(HalfParam, Half);
self_convert!(Q, Sample);
self_convert!(Resonance, Sample);
self_convert!(Unisolo, Half);
self_convert!(VibratoFreq, Half);
self_convert!(Pan, Sample);
self_convert!(Spread, Sample);
self_convert!(VoiceMode, Sample);

impl From<Note> for Freq {
    fn from(note: Note) -> Self {
        if note.frac() == 0 {
            NOTE_TAB[note.to_num::<usize>()].into()
        } else {
            note.frac()
                .lerp(
                    NOTE_TAB[note.to_num::<usize>()],
                    NOTE_TAB[note.to_num::<usize>() + 1],
                )
                .into()
        }
    }
}

// This is only accurate to the nearest octave
impl From<Freq> for Note {
    fn from(freq: Freq) -> Self {
        (((*freq * h!(0.00227272727273)).int_log2() * 12) + 69).into()
    }
}

impl From<Db> for Half {
    fn from(db: Db) -> Self {
        (*db * h!(0.166666666667)).exp2()
    }
}

impl From<Half> for Db {
    fn from(half: Half) -> Self {
        (half.int_log2() * h!(6)).into()
    }
}

impl From<EnvValue> for Sample {
    fn from(ev: EnvValue) -> Self {
        ((*ev - s!(1)) * s!(0.0002)).sqrt()
    }
}

impl From<Sample> for EnvValue {
    fn from(sample: Sample) -> Self {
        let half = Half::from_num(sample);
        Sample::from_num(half * half * h!(5000) + h!(1)).into()
    }
}

impl From<Volume> for Sample {
    fn from(v: Volume) -> Self {
        let v = *v * s!(4);
        v * v
    }
}

impl From<Sample> for Volume {
    fn from(sample: Sample) -> Self {
        (sample.sqrt() * s!(0.25)).into()
    }
}

impl From<Param> for bool {
    fn from(p: Param) -> Self {
        *p >= s!(0.5)
    }
}

impl From<bool> for Param {
    fn from(b: bool) -> Self {
        if b {
            s!(1).into()
        } else {
            s!(0).into()
        }
    }
}

//TODO: This needs optimising for data retention
impl From<Param> for Freq {
    fn from(p: Param) -> Self {
        (Half::from_num(*p * *p) * h!(19980) + h!(20)).into()
    }
}

//TODO: This needs optimising for data retention
impl From<Freq> for Param {
    fn from(f: Freq) -> Self {
        Sample::from_num((*f - h!(20)) * h!(0.0000500500500501))
            .sqrt()
            .into()
    }
}

// These have been optimised for speed
impl From<Param> for Q {
    fn from(p: Param) -> Self {
        if *p < s!(0.5) {
            (*p * s!(1.32) + s!(0.33)).into()
        } else {
            (*p * s!(22) - s!(10)).into()
        }
    }
}

impl From<Q> for Param {
    fn from(q: Q) -> Self {
        if *q < s!(1) {
            ((*q - s!(0.33)) * s!(0.757575757576)).into()
        } else {
            ((*q + s!(10)) * s!(0.0454545454545)).into()
        }
    }
}
