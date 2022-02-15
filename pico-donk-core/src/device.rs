use core::fmt;

use crate::cst::*;
use crate::helpers::*;

pub struct DeviceError {
    name: &'static str,
}

impl DeviceError {
    fn new<T: Device<A>, const A: usize>() -> DeviceError {
        DeviceError { name: T::NAME }
    }
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Device {} malfunctioned", self.name))
    }
}

pub trait Device<const NUM_PARAMS: usize> {
    const NAME: &'static str;
    type Param;
    fn run(&mut self, song_position: usize, buffer: &mut [Sample]) -> Result<usize, DeviceError>;
    fn set_param<T: Parameter>(&mut self, ty: Self::Param, value: T) -> ();
    fn get_param<T: Parameter>(&self, ty: Self::Param) -> T;
    fn set_chunk(&mut self, chunk: [i32; NUM_PARAMS]) -> ();
    fn get_chunk(&self) -> [i32; NUM_PARAMS];
}

pub trait SynthDevice<const NUM_PARAMS: usize>
where
    Self: Device<NUM_PARAMS>,
{
    const MAX_VOICES: usize = 8;
    const MAX_EVENTS: usize = 8;
    const MAX_ACTIVE_NOTES: usize = 4;
    type Voice;
    fn all_notes_off(&mut self);
    fn note_on(&mut self, note: Note, velocity: u32, delta_samples: usize);
    fn note_off(&mut self, note: Note, delta_samples: usize);

    fn get_voices_unisono(&self) -> Unisono;
    fn get_voices_detune(&self) -> Detune;
    fn get_voices_pan(&self) -> Pan;
    fn get_vibrato_freq(&self) -> VibratoFreq;
    fn get_vibrato_amount(&self) -> Sample;
    fn get_rise(&self) -> Sample;
    fn get_slide(&self) -> SlideTime;

    fn set_voices_unisono(&mut self, n: Unisono);
    fn set_voices_detune(&mut self, n: Detune);
    fn set_voices_pan(&mut self, n: Pan);
    fn set_vibrato_freq(&mut self, n: VibratoFreq);
    fn set_vibrato_amount(&mut self, n: Sample);
    fn set_rise(&mut self, n: Sample);
    fn set_slide(&mut self, n: SlideTime);

    fn clear_events(&mut self);
}

pub trait Voice {
    fn run(&self, song_position: usize, buffer: &mut [Sample]) -> Result<usize, DeviceError>;
    fn note_on(&mut self, note: Note, velocity: u32, detune: Detune, pan: Pan);
    fn note_off(&mut self);
    fn note_slide(&mut self, note: Note);

    fn is_on(&self) -> bool;
    fn get_note(&mut self) -> Note;
    fn get_detune(&self) -> Detune;
    fn get_pan(&self) -> Pan;
    fn get_vibrato_phase(&self) -> VibratoPhase;

    fn set_detune(&mut self, n: Detune);
    fn set_pan(&mut self, n: Pan);
    fn set_slide(&mut self, n: SlideTime);
    fn set_vibrato_phase(&mut self, n: VibratoPhase);
}

#[derive(PartialEq, Eq)]
pub enum EventType {
    None,
    NoteOn,
    NoteOff,
}

pub struct Event {
    pub ty: EventType,
    pub delta_samples: usize,
    pub note: Note,
    pub velocity: u32,
}
