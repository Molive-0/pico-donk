#![feature(concat_idents)]
#![feature(generic_const_exprs)]
mod device;
mod synth_device;

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use device::*;
use fixed::FixedI32;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::File;
use syn::Item;
use synth_device::*;

type Sample = FixedI32<24>;
type Half = FixedI32<16>;
type Quarter = FixedI32<8>;

#[proc_macro]
pub fn types(_: TokenStream) -> TokenStream {
    TokenStream::from(quote! {
    use fixed::FixedI32;
    pub type Sample = FixedI32<24>;
    pub type Half = FixedI32<16>;
    pub type Quarter = FixedI32<8>;
    })
}

#[proc_macro]
pub fn tables(_: TokenStream) -> TokenStream {
    let fast_sin_tab_log2_size: usize = 9; // size = 512
    let fast_sin_tab_size: usize = 1 << fast_sin_tab_log2_size;
    let adjusted_fast_sin_tab_size: usize = fast_sin_tab_size + 1;
    let fast_sin_tab = (0..adjusted_fast_sin_tab_size)
        .map(|f| {
            Sample::from_num(
                ((((f as f64) * (core::f64::consts::TAU / fast_sin_tab_size as f64)).sin() + 1.)
                    / 2.)
                    + 1.,
            )
            .to_bits()
        })
        .collect::<Vec<i32>>();

    let note_tab_size: usize = 128;
    let note_tab = (0..note_tab_size)
        .map(|f| Half::from_num(((f as f64 - 69.0) / 12.0).exp2() * 440.0).to_bits())
        .collect::<Vec<i32>>();

    let fast_exp_tab_log2_size: usize = 9; // size = 512
    let fast_exp_tab_size: usize = 1 << fast_exp_tab_log2_size;
    let adjusted_fast_exp_tab_size: usize = fast_exp_tab_size + 1;
    let fast_exp_tab = (0..adjusted_fast_exp_tab_size)
        .map(|f| Sample::from_num((f as f64 / fast_exp_tab_size as f64).exp2() - 1.).to_bits())
        .collect::<Vec<i32>>();

    let expanded = quote! {
        pub const FAST_SIN_TAB_LOG2_SIZE: usize = #fast_sin_tab_log2_size; // size = 512
        pub const FAST_SIN_TAB_SIZE: usize = #fast_sin_tab_size;
        pub const ADJUSTED_FAST_SIN_TAB_SIZE: usize = #adjusted_fast_sin_tab_size;
        pub const FAST_SIN_TAB: [Sample; #adjusted_fast_sin_tab_size] = [
            #(Sample::from_bits(#fast_sin_tab)),*
        ];
        pub const NOTE_TAB_SIZE: usize = #note_tab_size;
        pub const NOTE_TAB: [Sample; #note_tab_size] = [
            #(Sample::from_bits(#note_tab)),*
        ];
        pub const FAST_EXP_TAB_LOG2_SIZE: usize = #fast_exp_tab_log2_size; // size = 512
        pub const FAST_EXP_TAB_SIZE: usize = #fast_exp_tab_size;
        pub const ADJUSTED_FAST_EXP_TAB_SIZE: usize = #adjusted_fast_exp_tab_size;
        pub const FAST_EXP_TAB: [Sample; #adjusted_fast_exp_tab_size] = [
            #(Sample::from_bits(#fast_exp_tab)),*
        ];
    };

    // Hand the output tokens back to the compiler.
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn device(input: TokenStream) -> TokenStream {
    let mut ast: File = syn::parse(input.clone()).unwrap();
    //panic!("{:?}", ast);
    let (parameters_name, device_name) = {
        let mut ngv = NameGetVisitor::new();
        ngv.visit_file(&ast);
        ngv.find_device_name()
    };
    eprintln!("Creating device {}", device_name);
    let parameters = {
        let mut pv = ParameterVisitor::new(parameters_name.clone());
        pv.visit_file_mut(&mut ast);
        pv.parameters
    };
    {
        let mut dv = DeviceVisitor::new(device_name.clone(), parameters.len());
        dv.visit_file_mut(&mut ast);
    };
    let name = device_name.to_string();
    {
        let mut iv = ImplVisitor::new(name, parameters_name, parameters);
        iv.visit_file_mut(&mut ast);
    };

    ast.items
        .push(Item::Verbatim(quote! {use crate::helpers::Parameter;}));
    ast.items
        .push(Item::Verbatim(quote! {use crate::device::Device;}));

    ast.to_token_stream().into()
}

#[proc_macro]
pub fn synth_device(input: TokenStream) -> TokenStream {
    let mut ast: File = syn::parse(input.clone()).unwrap();
    //panic!("{:?}", ast);
    let (parameters_name, device_name, voice_name) = {
        let mut ngv = SynthNameGetVisitor::new();
        ngv.visit_file(&ast);
        ngv.find_device_name()
    };
    eprintln!("Creating synth device {}", device_name);
    let parameters = {
        let mut pv = ParameterVisitor::new(parameters_name.clone());
        pv.visit_file_mut(&mut ast);
        pv.parameters
    };
    {
        let mut dv =
            SynthDeviceVisitor::new(device_name.clone(), voice_name.clone(), parameters.len());
        dv.visit_file_mut(&mut ast);
    };
    {
        let mut vv = SynthVoiceVisitor::new(voice_name.clone(), parameters_name.clone());
        vv.visit_file_mut(&mut ast);
    };
    let name = device_name.to_string();
    {
        let mut iv = SynthImplVisitor::new(name.clone(), voice_name, parameters.len());
        iv.visit_file_mut(&mut ast);
    };

    ast.items
        .push(Item::Verbatim(quote! {use crate::helpers::*;}));
    ast.items.push(Item::Verbatim(quote! {use crate::cst::*;}));
    ast.items
        .push(Item::Verbatim(quote! {use crate::device::*;}));

    let len = parameters.len();
    let idents = parameters
        .iter()
        .map(|m| m.ident.clone().unwrap())
        .collect::<Vec<_>>();
    let types = parameters.iter().map(|m| m.ty.clone()).collect::<Vec<_>>();

    let run = get_run();

    ast.items.push(Item::Verbatim(quote! {
        impl Device<#len> for #device_name {
            const NAME: &'static str = #name;
            type Param = #parameters_name;
            fn set_param<T: Parameter>(&mut self, ty: Self::Param, value: T) {
                let loc = ty as usize;
                match ty {
                    #( #parameters_name::#idents => {
                        let temp: #types = value.into();
                        self._chunk_data[loc] = temp.into();
                    }, )*
                }
            }
            fn get_param<T: Parameter>(&self, ty: Self::Param) -> T {
                let loc = ty as usize;
                match ty {
                    #( #parameters_name::#idents => {
                        let temp: #types = self._chunk_data[loc].into();
                        temp.into()
                    }, )*
                }
            }
            fn set_chunk(&mut self, chunk: [i32; #len]) {
                self._chunk_data = chunk;
            }
            fn get_chunk(&self) -> [i32; #len]{
                self._chunk_data
            }
            #run
        }
    }));

    ast.to_token_stream().into()
}
