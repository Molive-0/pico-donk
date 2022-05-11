use core::panic;
use std::iter::once;

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::And;
use syn::token::{Colon, Gt, Lt, Pub};
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::TypeReference;
use syn::{
    visit, AngleBracketedGenericArguments, ExprLit, Field, Fields, GenericArgument, Ident,
    ImplItem, ItemImpl, ItemStruct, LitInt, PathArguments, Type, VisPublic, Visibility,
};

pub struct SynthNameGetVisitor {
    structs: Vec<Ident>,
}

impl SynthNameGetVisitor {
    pub fn new() -> SynthNameGetVisitor {
        SynthNameGetVisitor { structs: vec![] }
    }

    pub fn find_device_name(&mut self) -> (Ident, Ident, Ident) {
        if let Some(p) = self
            .structs
            .iter()
            .filter(|s| s.to_string().ends_with("Parameters"))
            .next()
        {
            let string = p.to_string();
            let n = string.strip_suffix("Parameters").unwrap();
            let m = n.to_owned() + "Voice";
            (
                p.clone(),
                self.structs
                    .iter()
                    .filter(|s| s.to_string() == n)
                    .next()
                    .expect(&format!("Main struct for {} not found", n))
                    .clone(),
                self.structs
                    .iter()
                    .filter(|s| s.to_string() == m)
                    .next()
                    .expect(&format!("Voice struct for {} not found", m))
                    .clone(),
            )
        } else {
            panic!("Parameters struct not found");
        }
    }
}

impl<'ast> Visit<'ast> for SynthNameGetVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        self.structs.push(node.ident.clone());

        // Delegate to the default impl to visit any nested functions.
        visit::visit_item_struct(self, node);
    }
}

macro_rules! fieldpub {
    ($f:ident, $name:expr, $ty:tt) => {
        $f.named.push(Field {
            attrs: vec![],
            vis: Visibility::Public(VisPublic {
                pub_token: Pub {
                    span: Span::call_site(),
                },
            }),
            ident: Some(Ident::new($name, Span::call_site())),
            colon_token: Some(Colon {
                spans: [Span::call_site()],
            }),
            ty: Type::Verbatim(quote! {$ty}),
        });
    };
}

macro_rules! field {
    ($f:ident, $name:expr, $ty:tt) => {
        $f.named.push(Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: Some(Ident::new($name, Span::call_site())),
            colon_token: Some(Colon {
                spans: [Span::call_site()],
            }),
            ty: Type::Verbatim(quote! {$ty}),
        });
    };
}
pub struct SynthDeviceVisitor {
    look_for: Ident,
    voice_name: Ident,
    length: usize,
}

impl SynthDeviceVisitor {
    pub fn new(look_for: Ident, voice_name: Ident, length: usize) -> SynthDeviceVisitor {
        SynthDeviceVisitor {
            look_for,
            voice_name,
            length,
        }
    }
}

impl VisitMut for SynthDeviceVisitor {
    fn visit_item_struct_mut(&mut self, node: &mut ItemStruct) {
        if node.ident == self.look_for {
            if let Fields::Named(f) = &mut node.fields {
                let len = self.length;
                field!(f, "_chunk_data", [i32; #len]);

                fieldpub!(f, "voices_unisono", Unisono);
                fieldpub!(f, "voices_detune", Detune);
                fieldpub!(f, "voices_pan", Pan);

                fieldpub!(f, "vibrato_freq", VibratoFreq);
                fieldpub!(f, "vibrato_amount", Sample);

                fieldpub!(f, "rise", Sample);
                fieldpub!(f, "slide", SlideTime);

                field!(f, "mono_active", bool);
                field!(f, "note_count", u8);

                field!(f, "note_log", [Note; Self::MAX_ACTIVE_NOTES]);
                field!(f, "active_notes", [bool; Self::MAX_ACTIVE_NOTES]);

                let name = &self.voice_name;
                field!(f, "voices", [#name; Self::MAX_VOICES]);
                field!(f, "events", [Event; Self::MAX_EVENTS]);
            } else {
                panic!("Struct {} in wrong format", self.look_for);
            }
        }
    }
}

pub struct SynthVoiceVisitor {
    look_for: Ident,
    parameters: Ident,
}

impl SynthVoiceVisitor {
    pub fn new(look_for: Ident, parameters: Ident) -> SynthVoiceVisitor {
        SynthVoiceVisitor {
            look_for,
            parameters,
        }
    }
}

impl VisitMut for SynthVoiceVisitor {
    fn visit_item_struct_mut(&mut self, node: &mut ItemStruct) {
        if node.ident == self.look_for {
            if let Fields::Named(f) = &mut node.fields {
                fieldpub!(f, "is_on", bool);
                fieldpub!(f, "note", Note);
                fieldpub!(f, "detune", Detune);
                fieldpub!(f, "pan", Pan);
                fieldpub!(f, "vibrato_phase", VibratoPhase);
                fieldpub!(f, "slide_time", SlideTime);
                field!(f, "slide_active", bool);
                field!(f, "slide_delta", Half);
                field!(f, "slide_samples", u32);
                field!(f, "destination_note", Note);
                field!(f, "current_note", Note);
                let param = &self.parameters;
                f.named.push(Field {
                    attrs: vec![],
                    vis: Visibility::Inherited,
                    ident: Some(Ident::new("parameters", Span::call_site())),
                    colon_token: Some(Colon {
                        spans: [Span::call_site()],
                    }),
                    ty: Type::Reference(TypeReference {
                        and_token: And {
                            spans: [Span::call_site()],
                        },
                        lifetime: None,
                        mutability: None,
                        elem: Box::new(Type::Verbatim(quote! {#param})),
                    }),
                });
            } else {
                panic!("Struct {} in wrong format", self.look_for);
            }
        }
    }
}

pub struct SynthImplVisitor {
    look_for: String,
    voice_name: Ident,
    len: usize,
}

impl SynthImplVisitor {
    pub fn new(look_for: String, voice_name: Ident, len: usize) -> SynthImplVisitor {
        SynthImplVisitor {
            look_for,
            voice_name,
            len,
        }
    }
}

macro_rules! variable {
    ($node:ident, $var:ident, $get:ident, $set:ident, $ty:ty) => {
        $node.items.push(ImplItem::Verbatim(quote! {
        fn $get(&self) -> $ty
            {
                self.$var
            }
        }));
        $node.items.push(ImplItem::Verbatim(quote! {
        fn $set(&mut self, n: $ty)
            {
                self.$var = n;
            }
        }));
    };
}

impl VisitMut for SynthImplVisitor {
    fn visit_item_impl_mut(&mut self, node: &mut ItemImpl) {
        if let Type::Path(p) = *node.self_ty.clone() {
            if node.trait_.is_some() {
                let path = p
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>();
                let path2 = node
                    .trait_
                    .as_ref()
                    .unwrap()
                    .1
                    .segments
                    .iter()
                    .map(|s| &s.ident)
                    .collect::<Vec<_>>();
                if path == [self.look_for.clone()] && path2 == ["SynthDevice"] {
                    let voice_name = self.voice_name.clone();
                    let len = self.len;
                    node.items.push(ImplItem::Verbatim(quote! {
                    type Voice = #voice_name;}));

                    variable!(
                        node,
                        voices_unisono,
                        get_voices_unisono,
                        set_voices_unisono,
                        Unisono
                    );
                    variable!(
                        node,
                        voices_detune,
                        get_voices_detune,
                        set_voices_detune,
                        Detune
                    );
                    variable!(node, voices_pan, get_voices_pan, set_voices_pan, Pan);
                    variable!(
                        node,
                        vibrato_freq,
                        get_vibrato_freq,
                        set_vibrato_freq,
                        VibratoFreq
                    );
                    variable!(
                        node,
                        vibrato_amount,
                        get_vibrato_amount,
                        set_vibrato_amount,
                        Sample
                    );
                    variable!(node, rise, get_rise, set_rise, Sample);
                    node.items.push(ImplItem::Verbatim(quote! {
                    fn get_slide(&self) -> SlideTime
                        {
                            self.slide
                        }
                    }));
                    node.items.push(ImplItem::Verbatim(quote! {
                    fn set_slide(&mut self, n: SlideTime)
                        {
                            self.slide = n;
                            for voice in self.voices.iter_mut() {
                                voice.set_slide(self.slide);
                            }
                        }
                    }));

                    node.items.push(ImplItem::Verbatim(quote! {
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
                    }));
                    node.items.push(ImplItem::Verbatim(quote! {
                        fn clear_events(&mut self) {
                            for event in self.events.iter_mut() {
                                event.ty = EventType::None;
                            }
                        }
                    }));
                    node.items.push(ImplItem::Verbatim(quote! {
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
                    }));
                    node.items.push(ImplItem::Verbatim(quote! {
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
                    }));

                    node.trait_
                        .as_mut()
                        .unwrap()
                        .1
                        .segments
                        .last_mut()
                        .as_mut()
                        .unwrap()
                        .arguments =
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: Lt {
                                spans: [Span::call_site()],
                            },
                            args: Punctuated::from_iter(once(GenericArgument::Const(
                                syn::Expr::Lit(ExprLit {
                                    attrs: vec![],
                                    lit: syn::Lit::Int(LitInt::new(
                                        &len.to_string(),
                                        Span::call_site(),
                                    )),
                                }),
                            ))),
                            gt_token: Gt {
                                spans: [Span::call_site()],
                            },
                        });
                } else if path == [self.voice_name.to_string()] && path2 == ["Voice"] {
                    node.items.push(ImplItem::Verbatim(quote! {
                        fn note_on(&mut self, note: Note, velocity: u32, detune: Detune, pan: Pan) {
                            self.is_on = true;
                            self.note = note;
                            self.current_note = note;
                            self.detune = detune;
                            self.pan = pan;
                            self.slide_active = false;
                        }
                    }));
                    node.items.push(ImplItem::Verbatim(quote! {
                        fn note_slide(&mut self, note: Note) {
                            self.slide_active = true;
                            self.destination_note = note;
                            self.slide_delta = ((*note-*self.current_note).molive_div(*self.slide_time)).into();
                            self.slide_samples = self.slide_time.to_num();
                        }
                    }));

                    node.items.push(ImplItem::Verbatim(quote! {
                        fn get_note(&mut self) -> Note{
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
                    }));

                    variable!(node, detune, get_detune, set_detune, Detune);
                    variable!(node, pan, get_pan, set_pan, Pan);
                    variable!(
                        node,
                        vibrato_phase,
                        get_vibrato_phase,
                        set_vibrato_phase,
                        VibratoPhase
                    );

                    node.items.push(ImplItem::Verbatim(quote! {
                        fn is_on(&self) -> bool {
                            self.is_on
                        }
                    }));
                    node.items.push(ImplItem::Verbatim(quote! {
                        fn set_slide(&mut self, n: SlideTime) {
                            self.slide_time = n;
                        }
                    }));
                }
            }
        }
    }
}

pub fn get_run() -> TokenStream {
    quote! {
        fn run(&mut self, mut song_position: usize, buffer: &mut [Sample]) -> Result<usize, DeviceError>
    {
        let mut num_samples = buffer.len();
        let mut position = 0;

        while num_samples > 0
        {
            let mut samples_to_next_event = num_samples;
            for e in self.events.iter_mut()
            {
                if e.ty != EventType::None
                {
                    if e.delta_samples == 0
                    {
                        match e.ty
                        {
                            EventType::NoteOn =>
                            {
                                let mut j = *self.voices_unisono;
                                for voice in self.voices.iter_mut()
                                {
                                    if !voice.is_on()
                                    {
                                        j-=1;
                                        let f = Sample::from_num(j) / (if *self.voices_unisono > 1 { *self.voices_unisono - 1 } else {1});
                                        voice.note_on(e.note, e.velocity, (f * *self.voices_detune).into(), ((f - sf!(0.5)) * (*self.voices_pan * s!(2) - s!(1)) + sf!(0.5)).into());
                                    }
                                    if j < 1 {break;}
                                }
                                e.ty = EventType::None;
                            },
                            EventType::NoteOff =>
                            {
                                for voice in self.voices.iter_mut()
                                {
                                    if voice.is_on() && voice.note == e.note {voice.note_off();}
                                }
                                e.ty = EventType::None;
                            },
                            EventType::None => {},
                        }
                    }
                    else if e.delta_samples < samples_to_next_event
                    {
                        samples_to_next_event = e.delta_samples;
                    }
                }
            }

            let output = &mut buffer[position..(samples_to_next_event+position)];

            for voice in self.voices.iter_mut()
            {
                if voice.is_on() {voice.run(song_position, output)?;}
            }
            for event in self.events.iter_mut()
            {
                if event.ty != EventType::None {event.delta_samples -= samples_to_next_event;}
            }
            song_position += samples_to_next_event / (crate::cst::SAMPLE_RATE as usize);
            position += samples_to_next_event;
            num_samples -= samples_to_next_event;
        }
        Ok(song_position)
    }
    }
}
