use pico_donk_proc_macro::synth_device;

synth_device!(
#[derive(Copy, Clone)]
struct FalconParameters {
    Test: i32
}

struct Falcon {}

struct FalconVoice {}

impl SynthDevice for Falcon {}

impl Voice for FalconVoice {
    fn note_off(&mut self) { }
    fn run(&self, song_position: usize, buffer: &mut [Sample]) -> Result<usize, DeviceError> { Ok(0) }}
);
