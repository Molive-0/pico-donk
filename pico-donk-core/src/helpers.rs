use core::ops::{Deref, DerefMut};

use fixed::{types::extra::LeEqU32, FixedI32};
use fixed_sqrt::FixedSqrt;

use crate::cst::*;

#[macro_export]
macro_rules! s { ($($a:tt)+) => { fixed_macro::fixed!($($a)+: I8F24) } }
#[macro_export]
macro_rules! h { ($($a:tt)+) => { fixed_macro::fixed!($($a)+: I16F16) } }
#[macro_export]
macro_rules! q { ($($a:tt)+) => { fixed_macro::fixed!($($a)+: I24F8) } }

pub trait ConstFromNum {
    fn const_from_num(x: f64) -> Self;
}

impl<T: LeEqU32> const ConstFromNum for FixedI32<T> {
    #[inline]
    fn const_from_num(x: f64) -> Self {
        Self::from_bits((x * (1 << Self::FRAC_NBITS) as f64) as i32)
    }
}

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

pub trait MoliveDiv<S> {
    fn molive_div(&self, x: FixedI32<S>) -> Self;
}

impl<T: LeEqU32, S: LeEqU32> const MoliveDiv<S> for FixedI32<T> {
    #[inline]
    fn molive_div(&self, x: FixedI32<S>) -> Self {
        Self::from_bits((self.to_bits() / x.to_bits()) << (FixedI32::<S>::FRAC_NBITS))
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

pub trait Exp {
    fn exp2(self) -> Self;
    fn exp10(self) -> Self;
}

impl Exp for Sample {
    fn exp2(self) -> Self {
        let scale: Self = if self > 0 {
            Self::from_bits(s!(1).to_bits() << self.floor().to_num::<i32>())
        } else {
            Self::from_bits(s!(1).to_bits() >> self.floor().abs().to_num::<i32>())
        };

        scale + (self.lookup(&FAST_EXP_TAB, FAST_EXP_TAB_LOG2_SIZE) * scale)
    }
    #[inline]
    fn exp10(self) -> Self {
        (self * s!(3.32192809489)).exp2()
    }
}

impl Exp for Half {
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
    #[inline]
    fn exp10(self) -> Self {
        (self * h!(3.32192809489)).exp2()
    }
}

pub trait SinCos {
    //Takes a number between 0 and 1 and returns a number between 0 and 1
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
        #[derive(Clone, Copy, Eq, PartialEq)]
        pub struct $name {
            value: $type,
        }
        impl const Deref for $name {
            type Target = $type;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.value
            }
        }
        impl const DerefMut for $name {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.value
            }
        }
        impl From<i32> for $name {
            #[inline]
            fn from(t: i32) -> Self {
                <$type>::from_num(t).into()
            }
        }
        impl From<$name> for i32 {
            #[inline]
            fn from(t: $name) -> Self {
                let t: $type = t.into();
                t.to_num()
            }
        }
    };
}
macro_rules! self_convert {
    ($name: ident, $type: ty) => {
        impl const From<$type> for $name {
            #[inline]
            fn from(t: $type) -> Self {
                $name { value: t }
            }
        }
        impl const From<$name> for $type {
            #[inline]
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
structs!(VibratoFreq, Half);
structs!(VibratoPhase, Half);
structs!(Pan, Sample);
structs!(Spread, Sample);
structs!(Detune, Sample);
structs!(SlideTime, Quarter);
self_convert!(Note, Half);
self_convert!(Freq, Half);
self_convert!(Param, Sample);
self_convert!(HalfParam, Half);
self_convert!(Q, Sample);
self_convert!(Resonance, Sample);
//self_convert!(Unisono, i32);
self_convert!(VibratoFreq, Half);
self_convert!(VibratoPhase, Half);
self_convert!(Pan, Sample);
self_convert!(Spread, Sample);
self_convert!(Detune, Sample);
self_convert!(SlideTime, Quarter);

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Unisono {
    value: i32,
}
impl const Deref for Unisono {
    type Target = i32;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl const DerefMut for Unisono {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl From<Note> for Freq {
    fn from(note: Note) -> Self {
        if note.frac() == 0i32 {
            Half::from_num(NOTE_TAB[note.to_num::<usize>()]).into()
        } else {
            Half::from_num(note.frac().lerp(
                NOTE_TAB[note.to_num::<usize>()],
                NOTE_TAB[note.to_num::<usize>() + 1],
            ))
            .into()
        }
    }
}

// This is only accurate to the nearest octave
impl From<Freq> for Note {
    #[inline]
    fn from(freq: Freq) -> Self {
        (((*freq * h!(0.00227272727273)).int_log2() * 12) + 69).into()
    }
}

impl From<Db> for Half {
    #[inline]
    fn from(db: Db) -> Self {
        (*db * h!(0.166666666667)).exp2()
    }
}

impl From<Half> for Db {
    #[inline]
    fn from(half: Half) -> Self {
        (half.int_log2() * h!(6)).into()
    }
}

impl From<EnvValue> for Sample {
    #[inline]
    fn from(ev: EnvValue) -> Self {
        ((*ev - s!(1)) * s!(0.0002)).sqrt()
    }
}

impl From<Sample> for EnvValue {
    #[inline]
    fn from(sample: Sample) -> Self {
        let half = Half::from_num(sample);
        Sample::from_num(half * half * h!(5000) + h!(1)).into()
    }
}

impl From<Volume> for Sample {
    #[inline]
    fn from(v: Volume) -> Self {
        let v = *v * s!(4);
        v * v
    }
}

impl From<Sample> for Volume {
    #[inline]
    fn from(sample: Sample) -> Self {
        (sample.sqrt() * s!(0.25)).into()
    }
}

pub trait Parameter
where
    Self: Into<Q>
        + From<Q>
        + Into<i32>
        + From<i32>
        + Into<bool>
        + From<bool>
        + Into<Freq>
        + From<Freq>
        + Into<SlideTime>
        + From<SlideTime>
        + Into<Note>
        + From<Note>,
{
}

impl Parameter for Param {}
//TODO
//impl Parameter for HalfParam {}

impl From<Param> for bool {
    #[inline]
    fn from(p: Param) -> Self {
        *p >= s!(0.5)
    }
}

impl From<bool> for Param {
    #[inline]
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
    #[inline]
    fn from(p: Param) -> Self {
        (Half::from_num(*p * *p) * h!(19980) + h!(20)).into()
    }
}

//TODO: This needs optimising for data retention
impl From<Freq> for Param {
    #[inline]
    fn from(f: Freq) -> Self {
        Sample::from_num((*f - h!(20)) * h!(0.0000500500500501))
            .sqrt()
            .into()
    }
}

// These have been optimised for speed
impl From<Param> for Q {
    #[inline]
    fn from(p: Param) -> Self {
        if *p < s!(0.5) {
            (*p * s!(1.32) + s!(0.33)).into()
        } else {
            (*p * s!(22) - s!(10)).into()
        }
    }
}

impl From<Q> for Param {
    #[inline]
    fn from(q: Q) -> Self {
        if *q < s!(1) {
            ((*q - s!(0.33)) * s!(0.757575757576)).into()
        } else {
            ((*q + s!(10)) * s!(0.0454545454545)).into()
        }
    }
}

impl From<Param> for SlideTime {
    #[inline]
    fn from(p: Param) -> Self {
        let m: Half = Half::from_num(*p);
        (Quarter::from_num(m * m * m * m) * q!(480000)).into()
    }
}

impl From<SlideTime> for Param {
    #[inline]
    fn from(q: SlideTime) -> Self {
        let m: Half = Half::from_num(*q / q!(480000));
        Sample::from_num(m.sqrt().sqrt()).into()
    }
}

impl From<HalfParam> for Note {
    #[inline]
    fn from(p: HalfParam) -> Self {
        (*p).into()
    }
}

impl From<Note> for HalfParam {
    #[inline]
    fn from(q: Note) -> Self {
        (*q).into()
    }
}

impl From<HalfParam> for Param {
    #[inline]
    fn from(p: HalfParam) -> Self {
        Sample::from_num(*p).into()
    }
}

impl From<Param> for HalfParam {
    #[inline]
    fn from(q: Param) -> Self {
        Half::from_num(*q).into()
    }
}

// WARNING: Imprecise
impl From<Param> for Note {
    #[inline]
    fn from(p: Param) -> Self {
        let t: HalfParam = p.into();
        t.into()
    }
}

// WARNING: Imprecise
impl From<Note> for Param {
    #[inline]
    fn from(q: Note) -> Self {
        let t: HalfParam = q.into();
        t.into()
    }
}

// Use an uncached version of a slice if we're working on arm
macro_rules! sampler_cache {
    ($x:expr) => {{
        #[cfg(target_arch = "arm")]
        unsafe {
            core::slice::from_raw_parts(
                (($x.as_ptr() as isize) & 0x00FFFFFF | 0x13000000) as *const u8,
                $x.len(),
            )
        }
        #[cfg(not(target_arch = "arm"))]
        $x
    }};
}
