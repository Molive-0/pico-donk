// constants
use build_const::build_const;
use fixed::types::I16F16;
use fixed::types::I3F29;

pub type Sample = I3F29;
pub type Half = I16F16;

pub const DRUMS: &'static [u8] = include_bytes!("../dat/cw_amen08_165.raw");
pub const LENGTH: usize = DRUMS.len() / 2;

// Constants created in the build script, where we had access to std
build_const!("cst");
