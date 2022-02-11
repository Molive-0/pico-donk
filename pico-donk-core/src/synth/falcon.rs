use crate::cst::Sample;
use crate::device::DeviceError;
use crate::helpers::Freq;
use pico_donk_proc_macro::device;

device!(
    #[derive(Copy, Clone, PartialEq, Eq)]
    struct FalconParameters {
        Test: Freq,
    }

    struct Falcon {}

    impl Device for Falcon {
        fn new() -> Falcon {
            Falcon {_chunkData: Default::default()}
        }
        fn run(&self, song_position: u32, input: Sample) -> Result<Sample, DeviceError> {
            Ok(s!(0))
        }
    }
);
