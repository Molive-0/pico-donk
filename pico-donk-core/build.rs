use build_const::ConstWriter;
use fixed::types::I3F29;
use std::f64::consts::TAU;

pub type Sample = I3F29;

macro_rules! sample {
    ($x:expr) => {
        format!("Sample::from_bits({})", $x.to_bits())
    };
}

fn main() {
    // use `for_build` in `build.rs`
    let consts = ConstWriter::for_build("cst").unwrap();

    // finish dependencies and starting writing constants
    let mut consts = consts.finish_dependencies();

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

    consts.add_value_raw("FP5", "Sample", &sample!(Sample::from_num(0.5)));

    consts.add_array_raw("FAST_SIN_TAB", "Sample", &fast_sin_tab);

    consts.finish();
}
