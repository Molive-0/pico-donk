mod falcon {
    use pico_donk_proc_macro::synth_device;
    enum FalconParameters {
        Test,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for FalconParameters {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for FalconParameters {
        #[inline]
        fn clone(&self) -> FalconParameters {
            {
                *self
            }
        }
    }
    struct Falcon {
        _chunk_data: [i32; 1usize],
        pub voices_unisono: Unisono,
        pub voices_detune: Detune,
        pub voices_pan: Pan,
        pub vibrato_freq: VibratoFreq,
        pub vibrato_amount: Sample,
        pub rise: Sample,
        pub slide: SlideTime,
        mono_active: bool,
        note_count: u8,
        note_log: [Note; Self::MAX_ACTIVE_NOTES],
        active_notes: [bool; Self::MAX_ACTIVE_NOTES],
        voices: [FalconVoice; Self::MAX_VOICES],
        events: [Event; Self::MAX_EVENTS],
    }
    struct FalconVoice {
        pub is_on: bool,
        pub note: Note,
        pub detune: Detune,
        pub pan: Pan,
        pub vibrato_phase: VibratoPhase,
        pub slide_time: SlideTime,
        slide_active: bool,
        slide_delta: Half,
        slide_samples: u32,
        destination_note: Note,
        current_note: Note,
    }
    impl SynthDevice<1> for Falcon {
        type Voice = FalconVoice;
        fn get_voices_unisono(&self) -> Unisono {
            self.voices_unisono
        }
        fn set_voices_unisono(&mut self, n: Unisono) {
            self.voices_unisono = n
        }
        fn get_voices_detune(&self) -> Detune {
            self.voices_detune
        }
        fn set_voices_detune(&mut self, n: Detune) {
            self.voices_detune = n
        }
        fn get_voices_pan(&self) -> Pan {
            self.voices_pan
        }
        fn set_voices_pan(&mut self, n: Pan) {
            self.voices_pan = n
        }
        fn get_vibrato_freq(&self) -> VibratoFreq {
            self.vibrato_freq
        }
        fn set_vibrato_freq(&mut self, n: VibratoFreq) {
            self.vibrato_freq = n
        }
        fn get_vibrato_amount(&self) -> Sample {
            self.vibrato_amount
        }
        fn set_vibrato_amount(&mut self, n: Sample) {
            self.vibrato_amount = n
        }
        fn get_rise(&self) -> Sample {
            self.rise
        }
        fn set_rise(&mut self, n: Sample) {
            self.rise = n
        }
        fn get_slide(&self) -> SlideTime {
            self.slide
        }
        fn set_slide(&mut self, n: SlideTime) {
            self.slide = n;
            for voice in self.voices.iter_mut() {
                voice.set_slide(self.slide);
            }
        }
        fn all_notes_off(&mut self) {
            for voice in self.voices.iter_mut() {
                if voice.is_on() {
                    voice.note_off();
                }
            }
            self.mono_active = false;
            self.note_count = 0;
            for note in self.active_notes.iter_mut() {
                *note = false;
            }
            self.clear_events();
        }
        fn clear_events(&mut self) {
            for event in self.events.iter_mut() {
                event.ty = EventType::None;
            }
        }
        fn note_on(&mut self, note: Note, velocity: u32, delta_samples: usize) {
            for event in self.events.iter_mut() {
                if event.ty == EventType::None {
                    event.ty = EventType::NoteOn;
                    event.delta_samples = delta_samples;
                    event.note = note;
                    event.velocity = velocity;
                    break;
                }
            }
        }
        fn note_off(&mut self, note: Note, delta_samples: usize) {
            for event in self.events.iter_mut() {
                if event.ty == EventType::None {
                    event.ty = EventType::NoteOff;
                    event.delta_samples = delta_samples;
                    event.note = note;
                    break;
                }
            }
        }
    }
    impl Voice for FalconVoice {
        fn note_off(&mut self) {}
        fn run(&self, song_position: usize, buffer: &mut [Sample]) -> Result<usize, DeviceError> {
            Ok(0)
        }
        fn note_on(&mut self, note: Note, velocity: u32, detune: Detune, pan: Pan) {
            self.is_on = true;
            self.note = note;
            self.current_note = note;
            self.detune = detune;
            self.pan = pan;
            self.slide_active = false;
        }
        fn note_slide(&mut self, note: Note) {
            self.slide_active = true;
            self.destination_note = note;
            self.slide_delta = ((*note - *self.current_note).molive_div(*self.slide_time)).into();
            self.slide_samples = self.slide_time.to_num();
        }
        fn get_note(&mut self) -> Note {
            if self.slide_active {
                self.current_note = (*self.current_note + self.slide_delta).into();
                if self.slide_samples == 0 {
                    self.note = self.destination_note;
                    self.current_note = self.destination_note;
                    self.slide_active = false;
                }
                self.slide_samples = self.slide_samples.wrapping_sub(1);
            }
            self.current_note
        }
        fn get_detune(&self) -> Detune {
            self.detune
        }
        fn set_detune(&mut self, n: Detune) {
            self.detune = n
        }
        fn get_pan(&self) -> Pan {
            self.pan
        }
        fn set_pan(&mut self, n: Pan) {
            self.pan = n
        }
        fn get_vibrato_phase(&self) -> VibratoPhase {
            self.vibrato_phase
        }
        fn set_vibrato_phase(&mut self, n: VibratoPhase) {
            self.vibrato_phase = n
        }
        fn is_on(&self) -> bool {
            self.is_on
        }
        fn set_slide(&mut self, n: SlideTime) {
            self.slide_time = n;
        }
    }
    use crate::helpers::*;
    use crate::cst::*;
    use crate::device::*;
    impl Device<1usize> for Falcon {
        const NAME: &'static str = "Falcon";
        type Param = FalconParameters;
        fn set_param<T: Parameter>(&mut self, ty: Self::Param, value: T) {
            let loc = ty as usize;
            match ty {
                FalconParameters::Test => {
                    let temp: i32 = value.into();
                    self._chunk_data[loc] = temp.into();
                }
            }
        }
        fn get_param<T: Parameter>(&self, ty: Self::Param) -> T {
            let loc = ty as usize;
            match ty {
                FalconParameters::Test => {
                    let temp: i32 = self._chunk_data[loc].into();
                    temp.into()
                }
            }
        }
        fn set_chunk(&mut self, chunk: [i32; 1usize]) {
            self._chunk_data = chunk;
        }
        fn get_chunk(&self) -> [i32; 1usize] {
            self._chunk_data
        }
        fn run(
            &mut self,
            mut song_position: usize,
            buffer: &mut [Sample],
        ) -> Result<usize, DeviceError> {
            let mut num_samples = buffer.len();
            let mut position = 0;
            while num_samples > 0 {
                let mut samples_to_next_event = num_samples;
                for e in self.events.iter() {
                    if e.ty != EventType::None {
                        if e.delta_samples == 0 {
                            match e.ty {
                                EventType::NoteOn => {
                                    let mut j = *self.voices_unisono;
                                    for voice in self.voices.iter_mut() {
                                        if !voice.is_on() {
                                            j -= 1;
                                            if j < 1 {
                                                break;
                                            }
                                            let f = Sample::from_num(j)
                                                / (if *self.voices_unisono > 1 {
                                                    *self.voices_unisono - 1
                                                } else {
                                                    1
                                                });
                                            voice . note_on (e . note , e . velocity , (f * * self . voices_detune) . into () , ((f - :: fixed_macro :: __fixed :: types :: I8F24 :: from_bits (8388608)) * (* self . voices_pan * :: fixed_macro :: __fixed :: types :: I8F24 :: from_bits (33554432) - :: fixed_macro :: __fixed :: types :: I8F24 :: from_bits (16777216)) + :: fixed_macro :: __fixed :: types :: I8F24 :: from_bits (8388608)) . into ()) ;
                                        }
                                    }
                                }
                                EventType::NoteOff => {
                                    for voice in self.voices.iter_mut() {
                                        if voice.is_on() && voice.note == e.note {
                                            voice.note_off();
                                        }
                                    }
                                }
                                EventType::None => {}
                            }
                        } else if e.delta_samples < samples_to_next_event {
                            samples_to_next_event = e.delta_samples;
                        }
                    }
                }
                let output = &mut buffer[position..(samples_to_next_event - position)];
                for voice in self.voices.iter_mut() {
                    if voice.is_on() {
                        voice.run(song_position, output)?;
                    }
                }
                for event in self.events.iter_mut() {
                    if event.ty != EventType::None {
                        event.delta_samples -= samples_to_next_event;
                    }
                }
                song_position += samples_to_next_event / (crate::cst::SAMPLE_RATE as usize);
                position += samples_to_next_event;
                num_samples -= samples_to_next_event;
            }
            Ok(song_position)
        }
    }
}
