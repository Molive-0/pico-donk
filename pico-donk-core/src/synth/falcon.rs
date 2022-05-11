use crate::envelope::Envelope;
use pico_donk_proc_macro::synth_device;

synth_device!(
#[derive(Copy, Clone)]
pub struct FalconParameters {
    Osc1Waveform: Param,
    Osc1RatioCoarse: Param,
    Osc1RatioFine: Param,
    Osc1Feedback: Param,
    Osc1FeedForward: Param,

    Osc1Attack: EnvValue,
    Osc1Decay: EnvValue,
    Osc1Sustain: Param,
    Osc1Release: EnvValue,

    Osc2Waveform: Param,
    Osc2RatioCoarse: Param,
    Osc2RatioFine: Param,
    Osc2Feedback: Param,

    Osc2Attack: EnvValue,
    Osc2Decay: EnvValue,
    Osc2Sustain: Param,
    Osc2Release: EnvValue,

    MasterLevel: Param,

    PitchAttack: EnvValue,
    PitchDecay: EnvValue,
    PitchSustain: Param,
    PitchRelease: EnvValue,
    PitchEnvAmt1: FalconEnvAmount,
    PitchEnvAmt2: FalconEnvAmount,
}

#[derive(Debug, Default)]
pub struct Falcon {}

impl Falcon {
    fn new() -> Self {
        let mut falcon: Falcon = Default::default();
        value!(falcon, FalconParameters::Osc1Waveform) = s!(0).to_bits();
        value!(falcon, FalconParameters::Osc1RatioCoarse) = s!(0).to_bits();
        value!(falcon, FalconParameters::Osc1RatioFine) = sf!(0.5).to_bits();
        value!(falcon, FalconParameters::Osc1Feedback) = s!(0).to_bits();
        value!(falcon, FalconParameters::Osc1FeedForward) = s!(0).to_bits();

        value!(falcon, FalconParameters::Osc1Attack) = s!(1).to_bits();
        value!(falcon, FalconParameters::Osc1Decay) = s!(1).to_bits();
        value!(falcon, FalconParameters::Osc1Sustain) = s!(1).to_bits();
        value!(falcon, FalconParameters::Osc1Release) = s!(1).to_bits();

        value!(falcon, FalconParameters::Osc2Waveform) = s!(0).to_bits();
        value!(falcon, FalconParameters::Osc2RatioCoarse) = s!(0).to_bits();
        value!(falcon, FalconParameters::Osc2RatioFine) = sf!(0.5).to_bits();
        value!(falcon, FalconParameters::Osc2Feedback) = s!(0).to_bits();

        value!(falcon, FalconParameters::Osc1Attack) = s!(1).to_bits();
        value!(falcon, FalconParameters::Osc1Decay) = s!(5).to_bits();
        value!(falcon, FalconParameters::Osc1Sustain) = sf!(0.75).to_bits();
        value!(falcon, FalconParameters::Osc1Release) = sf!(1.5).to_bits();

        value!(falcon, FalconParameters::MasterLevel) = sf!(0.8).to_bits();

        value!(falcon, FalconParameters::PitchAttack) = s!(1).to_bits();
        value!(falcon, FalconParameters::PitchDecay) = s!(5).to_bits();
        value!(falcon, FalconParameters::PitchSustain) = sf!(0.5).to_bits();
        value!(falcon, FalconParameters::PitchRelease) = sf!(1.5).to_bits();
        value!(falcon, FalconParameters::PitchEnvAmt1) = s!(0).to_bits();
        value!(falcon, FalconParameters::PitchEnvAmt2) = s!(0).to_bits();
        defaults!(falcon);
        falcon
    }
}

#[derive(Debug, Default, Clone)]
pub struct FalconVoice {
    osc1Env: Envelope,
    osc2Env: Envelope,
    pitchEnv: Envelope,

    osc1Phase: Sample,
    osc2Phase: Sample,

    osc1Output: Sample,
    osc2Output: Sample,
}

impl SynthDevice for Falcon {}

impl Voice for FalconVoice {
    fn note_on(&mut self) {
        self.osc1Phase = Sample::rand();
        self.osc2Phase = self.osc1Phase;
        self.osc1Env.set_attack(Sample::from_bits(value!(self.parameters, FalconParameters::Osc1Attack)));
        /*self.osc1Env.decay = falcon->osc1Decay;
        self.osc1Env.Sustain = falcon->osc1Sustain;
        self.osc1Env.Release = falcon->osc1Release;
        self.osc1Env.Trigger();
        self.osc2Env.Attack = falcon->osc2Attack;
        self.osc2Env.Decay = falcon->osc2Decay;
        self.osc2Env.Sustain = falcon->osc2Sustain;
        self.osc2Env.Release = falcon->osc2Release;
        self.osc2Env.Trigger();

        self.pitchEnv.Attack = falcon->pitchAttack;
        self.pitchEnv.Decay = falcon->pitchDecay;
        self.pitchEnv.Sustain = falcon->pitchSustain;
        self.pitchEnv.Release = falcon->pitchRelease;
        self.pitchEnv.Trigger();

        osc1Output = osc2Output = 0.0;*/
    }
    fn note_off(&mut self) { }
    fn run(&self, song_position: usize, buffer: &mut [Sample]) -> Result<usize, DeviceError> {
        for i in buffer {
            *i += s!(1);
        }
        Ok(0) }}
);
