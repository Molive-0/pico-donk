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
    fn new() -> Self;
    fn run(&self, song_position: u32, input: Sample) -> Result<Sample, DeviceError>;
    fn set_param<T: Parameter>(&mut self, ty: Self::Param, value: T) -> ();
    fn get_param<T: Parameter>(&self, ty: Self::Param) -> T;
}

pub trait SynthDevice<const NUM_PARAMS: usize>
where
    Self: Device<NUM_PARAMS>,
{
    fn all_notes_off();
    fn note_on(note: Note, velocity: u32);
    fn note_off(note: Note);
    const MAX_VOICES: usize = 8;
    const MAX_EVENTS: usize = 8;
    const MAX_ACTIVE_NOTES: usize = 4;
}

pub trait Voice {
    fn new<T: Device<A>, const A: usize>(parameters: &[Param; A]) -> Self;
    fn run(song_position: u32) -> Result<Sample, DeviceError>;
    fn note_on(note: Note, velocity: u32, detune: Detune);
    fn note_off(note: Note);
    fn note_slide(note: Note);

    fn is_on() -> bool;
    fn get_note() -> Note;
    fn get_detune() -> Detune;
    fn get_vibrato_phase() -> VibratoPhase;
}

pub enum EventType {
    None,
    NoteOn,
    NoteOff,
}
