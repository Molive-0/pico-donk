use pico_donk_proc_macro::synth_device;

synth_device!(
#[derive(Copy, Clone)]
pub struct FalconParameters {
    Test: i32
}

#[derive(Debug, Default)]
pub struct Falcon {}

#[derive(Debug, Default)]
pub struct FalconVoice {}

impl SynthDevice for Falcon {}

impl Voice for FalconVoice {
    fn note_off(&mut self) { }
    fn run(&self, song_position: usize, buffer: &mut [Sample]) -> Result<usize, DeviceError> {
        for i in buffer {
            *i += s!(1);
        }
        Ok(0) }}
);
