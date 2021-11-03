use build_const::ConstWriter;
use fixed::types::I16F16;
use fixed::types::I8F24;
use std::f64::consts::TAU;

pub type Sample = I8F24;
pub type Half = I16F16;

macro_rules! sample {
    ($x:expr) => {
        format!("Sample::from_bits({})", $x.to_bits())
    };
}

macro_rules! half {
    ($x:expr) => {
        format!("Half::from_bits({})", $x.to_bits())
    };
}

fn main() {
    // use `for_build` in `build.rs`
    let consts = ConstWriter::for_build("cst").unwrap();

    // finish dependencies and starting writing constants
    let mut consts = consts.finish_dependencies();

    // Sin table
    {
        let fast_sin_tab_log2_size: usize = 9; // size = 512
        let fast_sin_tab_size: usize = 1 << fast_sin_tab_log2_size;
        let adjusted_fast_sin_tab_size: usize = fast_sin_tab_size + 1;
        let fast_sin_tab: Vec<String> = (0..adjusted_fast_sin_tab_size)
            .map(|f| {
                sample!(Sample::from_num(
                    ((((f as f64) * (TAU / fast_sin_tab_size as f64)).sin() + 1.) / 2.) + 1.
                ))
            })
            .collect();

        consts.add_value("FAST_SIN_TAB_LOG2_SIZE", "usize", fast_sin_tab_log2_size);
        consts.add_value("FAST_SIN_TAB_SIZE", "usize", fast_sin_tab_size);
        consts.add_value(
            "ADJUSTED_FAST_SIN_TAB_SIZE",
            "usize",
            adjusted_fast_sin_tab_size,
        );

        consts.add_array_raw("FAST_SIN_TAB", "Sample", &fast_sin_tab);
    }

    // Note table
    {
        let note_tab_size = 128;
        let note_tab: Vec<String> = (0..note_tab_size)
            .map(|f| half!(Half::from_num(((f as f64 - 69.0) / 12.0).exp2() * 440.0)))
            .collect();

        consts.add_value("NOTE_TAB_SIZE", "usize", note_tab_size);
        consts.add_array_raw("NOTE_TAB", "Half", &note_tab);
    }

    // Exp2 table
    {
        let fast_exp_tab_log2_size: usize = 9; // size = 512
        let fast_exp_tab_size: usize = 1 << fast_exp_tab_log2_size;
        let adjusted_fast_exp_tab_size: usize = fast_exp_tab_size + 1;
        let fast_exp_tab: Vec<String> = (0..adjusted_fast_exp_tab_size)
            .map(|f| {
                sample!(Sample::from_num(
                    (f as f64 / fast_exp_tab_size as f64).exp2() - 1.
                ))
            })
            .collect();

        consts.add_value("FAST_EXP_TAB_LOG2_SIZE", "usize", fast_exp_tab_log2_size);
        consts.add_value("FAST_EXP_TAB_SIZE", "usize", fast_exp_tab_size);
        consts.add_value(
            "ADJUSTED_FAST_EXP_TAB_SIZE",
            "usize",
            adjusted_fast_exp_tab_size,
        );

        consts.add_array_raw("FAST_EXP_TAB", "Sample", &fast_exp_tab);
    }

    consts.finish();
}
