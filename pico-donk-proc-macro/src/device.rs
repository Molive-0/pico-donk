use core::panic;
use std::iter::once;

use proc_macro2::Span;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::token::{Bracket, Colon, Gt, Lt, Semi};
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{
    visit, AngleBracketedGenericArguments, Expr, ExprLit, Field, GenericArgument, Ident, ImplItem,
    Item, ItemEnum, ItemImpl, LitInt, PathArguments, Type, TypeArray, Variant, Visibility,
};
use syn::{Fields, ItemStruct};

pub struct NameGetVisitor {
    structs: Vec<Ident>,
}

impl NameGetVisitor {
    pub fn new() -> NameGetVisitor {
        NameGetVisitor { structs: vec![] }
    }

    pub fn find_device_name(&mut self) -> (Ident, Ident) {
        if let Some(p) = self
            .structs
            .iter()
            .filter(|s| s.to_string().ends_with("Parameters"))
            .next()
        {
            let string = p.to_string();
            let n = string.strip_suffix("Parameters").unwrap();
            (
                p.clone(),
                self.structs
                    .iter()
                    .filter(|s| s.to_string() == n)
                    .next()
                    .expect(&format!("Main struct for {} not found", n))
                    .clone(),
            )
        } else {
            panic!("Parameters struct not found");
        }
    }
}

impl<'ast> Visit<'ast> for NameGetVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        self.structs.push(node.ident.clone());

        // Delegate to the default impl to visit any nested functions.
        visit::visit_item_struct(self, node);
    }
}

pub struct ParameterVisitor {
    look_for: Ident,
    pub parameters: Vec<Field>,
}

impl ParameterVisitor {
    pub fn new(look_for: Ident) -> ParameterVisitor {
        ParameterVisitor {
            look_for,
            parameters: vec![],
        }
    }
}

impl VisitMut for ParameterVisitor {
    fn visit_item_mut(&mut self, node: &mut Item) {
        if let Item::Struct(item) = node {
            if item.ident == self.look_for {
                let brace;
                if let Fields::Named(f) = &item.fields {
                    self.parameters.extend(f.named.iter().cloned());
                    brace = f.brace_token;
                } else {
                    panic!("Struct {} in wrong format", self.look_for);
                }
                let _enum = ItemEnum {
                    attrs: item.attrs.clone(),
                    vis: item.vis.clone(),
                    enum_token: syn::token::Enum {
                        span: item.struct_token.span,
                    },
                    ident: item.ident.clone(),
                    generics: item.generics.clone(),
                    brace_token: brace,
                    variants: Punctuated::from_iter(self.parameters.iter().map(|p| Variant {
                        attrs: p.attrs.clone(),
                        ident: p.ident.clone().unwrap(),
                        fields: Fields::Unit,
                        discriminant: None,
                    })),
                };
                *node = Item::Enum(_enum);
            }
        }
    }
}

pub struct DeviceVisitor {
    look_for: Ident,
    length: usize,
}

impl DeviceVisitor {
    pub fn new(look_for: Ident, length: usize) -> DeviceVisitor {
        DeviceVisitor { look_for, length }
    }
}

impl VisitMut for DeviceVisitor {
    fn visit_item_struct_mut(&mut self, node: &mut ItemStruct) {
        if node.ident == self.look_for {
            if let Fields::Named(f) = &mut node.fields {
                f.named.push(Field {
                    attrs: vec![],
                    vis: Visibility::Inherited,
                    ident: Some(Ident::new("_chunkData", Span::call_site())),
                    colon_token: Some(Colon {
                        spans: [Span::call_site()],
                    }),
                    ty: Type::Array(TypeArray {
                        bracket_token: Bracket {
                            span: Span::call_site(),
                        },
                        elem: Box::new(Type::Verbatim(
                            Ident::new("i32", Span::call_site()).to_token_stream(),
                        )),
                        semi_token: Semi {
                            spans: [Span::call_site()],
                        },
                        len: Expr::Lit(ExprLit {
                            attrs: vec![],
                            lit: syn::Lit::Int(LitInt::new(
                                &self.length.to_string(),
                                Span::call_site(),
                            )),
                        }),
                    }),
                });
            } else {
                panic!("Struct {} in wrong format", self.look_for);
            }
        }
    }
}

pub struct ImplVisitor {
    look_for: String,
    parameters_name: Ident,
    parameters: Vec<Field>,
}

impl ImplVisitor {
    pub fn new(look_for: String, parameters_name: Ident, parameters: Vec<Field>) -> ImplVisitor {
        ImplVisitor {
            look_for,
            parameters_name,
            parameters,
        }
    }
}

impl VisitMut for ImplVisitor {
    fn visit_item_impl_mut(&mut self, node: &mut ItemImpl) {
        if let Type::Path(p) = *node.self_ty.clone() {
            if p.path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                == [self.look_for.clone()]
                && node.trait_.is_some()
            {
                if node
                    .trait_
                    .as_ref()
                    .unwrap()
                    .1
                    .segments
                    .iter()
                    .map(|s| &s.ident)
                    .collect::<Vec<_>>()
                    == ["Device"]
                {
                    let name = self.look_for.clone();
                    let parameters_name = self.parameters_name.clone();
                    let len = self.parameters.len();
                    let idents = self
                        .parameters
                        .iter()
                        .map(|m| m.ident.clone().unwrap())
                        .collect::<Vec<_>>();
                    let types = self
                        .parameters
                        .iter()
                        .map(|m| m.ty.clone())
                        .collect::<Vec<_>>();

                    node.items.push(ImplItem::Verbatim(quote! {
                    const NAME: &'static str = #name;}));
                    node.items.push(ImplItem::Verbatim(quote! {
                    type Param = #parameters_name;}));
                    node.items.push(ImplItem::Verbatim(quote! {
                    fn set_param<T: Parameter>(&mut self, ty: Self::Param, value: T) {
                        let loc = ty as usize;
                        match ty {
                            #( #parameters_name::#idents => {
                                let temp: #types = value.into();
                                self._chunkData[loc] = temp.into();
                            }, )*
                        }
                    }}));
                    node.items.push(ImplItem::Verbatim(quote! {
                    fn get_param<T: Parameter>(&self, ty: Self::Param) -> T {
                        let loc = ty as usize;
                        match ty {
                            #( #parameters_name::#idents => {
                                let temp: #types = self._chunkData[loc].into();
                                temp.into()
                            }, )*
                        }
                    }}));

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
                }
            }
        }
    }
}
