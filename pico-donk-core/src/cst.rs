// constants
use pico_donk_proc_macro::{tables, types};

pub const SAMPLE_RATE: i32 = 48000;

types!();
tables!();

pub const DRUMS: &'static [u8] = include_bytes!("../dat/cw_amen08_165.raw");
pub const LENGTH: usize = DRUMS.len() / 2;
