use pico_donk_proc_macro::synth_device;

synth_device!(
#[derive(Copy, Clone)]
pub struct FalconParameters {
    Osc1Waveform: Sample,
    Osc1RatioCoarse: Sample,
    Osc1RatioFine: Sample,
    Osc1Feedback: Sample,
    Osc1FeedForward: Sample,

    Osc1Attack: EnvValue,
    Osc1Decay: EnvValue,
    Osc1Sustain: Sample,
    Osc1Release: EnvValue,

    Osc2Waveform: Sample,
    Osc2RatioCoarse: Sample,
    Osc2RatioFine: Sample,
    Osc2Feedback: Sample,

    Osc2Attack: EnvValue,
    Osc2Decay: EnvValue,
    Osc2Sustain: Sample,
    Osc2Release: EnvValue,

    MasterLevel: Sample,

    PitchAttack: EnvValue,
    PitchDecay: EnvValue,
    PitchSustain: Sample,
    PitchRelease: EnvValue,
    PitchEnvAmt1: FalconEnvAmount,
    PitchEnvAmt2: FalconEnvAmount,
}

#[derive(Debug, Default)]
pub struct Falcon {
    fn new() -> Self {
        let mut falcon: Falcon = Default::default();
        value!(falcon, Osc1Waveform) = s!(0).into();
        value!(falcon, Osc1RatioCoarse) = s!(0).into();
        value!(falcon, Osc1RatioFine) = sf!(0.5).into();
        value!(falcon, Osc1Feedback) = s!(0).into();
        value!(falcon, Osc1FeedForward) = s!(0).into();

        value!(falcon, Osc1Attack) = s!(1).into();
        value!(falcon, Osc1Decay) = s!(1).into();
        value!(falcon, Osc1Sustain) = s!(1).into();
        value!(falcon, Osc1Release) = s!(1).into();

        value!(falcon, Osc2Waveform) = s!(0).into();
        value!(falcon, Osc2RatioCoarse) = s!(0).into();
        value!(falcon, Osc2RatioFine) = sf!(0.5).into();
        value!(falcon, Osc2Feedback) = s!(0).into();

        value!(falcon, Osc1Attack) = s!(1).into();
        value!(falcon, Osc1Decay) = s!(5).into();
        value!(falcon, Osc1Sustain) = sf!(0.75).into();
        value!(falcon, Osc1Release) = sf!(1.5).into();

        value!(falcon, MasterLevel) = sf!(.8).into();

        value!(falcon, PitchAttack) = s!(1).into();
        value!(falcon, PitchDecay) = s!(5).into();
        value!(falcon, PitchSustain) = sf!(0.5).into();
        value!(falcon, PitchRelease) = sf!(1.5).into();
        value!(falcon, PitchEnvAmt1) = sf!(0).into();
        value!(falcon, PitchEnvAmt2) = sf!(0).into();
        falcon
    }
}

#[derive(Debug, Default, Clone)]
pub struct FalconVoice {
}

impl SynthDevice for Falcon {}

impl Voice for FalconVoice {
    fn note_off(&mut self) { }
    fn run(&self, song_position: usize, buffer: &mut [Sample]) -> Result<usize, DeviceError> {
        for i in buffer {
            *i += s!(1);
        }
        Ok(0) }}
);
