// constants
use build_const::build_const;
use fixed::types::I16F16;
use fixed::types::I8F24;

pub type Sample = I8F24;
pub type Half = I16F16;

#[macro_export]
macro_rules! s { ($($a:tt)+) => { fixed_macro::fixed!($($a)+: I8F24) } }
#[macro_export]
macro_rules! h { ($($a:tt)+) => { fixed_macro::fixed!($($a)+: I16F16) } }

pub const DRUMS: &'static [u8] = include_bytes!("../dat/cw_amen08_165.raw");
pub const LENGTH: usize = DRUMS.len() / 2;

// Constants created in the build script, where we had access to std
build_const!("cst");
