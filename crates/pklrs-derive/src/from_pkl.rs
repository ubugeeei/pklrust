use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Lit};

pub(crate) fn impl_from_pkl(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("FromPkl only supports structs with named fields"),
        },
        _ => panic!("FromPkl only supports structs"),
    };

    let mut serde_field_attrs = Vec::new();
    for field in fields.iter() {
        let mut attrs = Vec::new();
        for attr in &field.attrs {
            if !attr.path().is_ident("pkl") {
                continue;
            }
            let result = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value = meta.value()?;
                    let lit: Lit = value.parse()?;
                    if let Lit::Str(lit_str) = lit {
                        let val = lit_str.value();
                        attrs.push(quote! { #[serde(rename = #val)] });
                    }
                } else if meta.path.is_ident("default") {
                    if let Ok(value) = meta.value() {
                        let lit: Lit = value.parse().expect("expected string literal");
                        if let Lit::Str(lit_str) = lit {
                            let val = lit_str.value();
                            attrs.push(quote! { #[serde(default = #val)] });
                        }
                    } else {
                        attrs.push(quote! { #[serde(default)] });
                    }
                }
                Ok(())
            });
            if let Err(e) = result {
                return e.to_compile_error();
            }
        }
        serde_field_attrs.push(attrs);
    }

    let field_defs: Vec<_> = fields
        .iter()
        .zip(serde_field_attrs.iter())
        .map(|(field, attrs)| {
            let ident = &field.ident;
            let ty = &field.ty;
            let vis = &field.vis;
            quote! {
                #(#attrs)*
                #vis #ident: #ty
            }
        })
        .collect();

    let field_assignments: Vec<_> = fields
        .iter()
        .map(|field| {
            let ident = &field.ident;
            quote! { #ident: __shadow.#ident }
        })
        .collect();

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Deserialize from a `pklrs::PklValue`.
            pub fn from_pkl_value(value: &::pklrs::PklValue) -> ::std::result::Result<Self, ::pklrs::Error> {
                #[derive(::serde::Deserialize)]
                struct __Shadow #ty_generics #where_clause {
                    #(#field_defs,)*
                }

                let __shadow: __Shadow = ::pklrs::from_pkl_value(value)?;
                Ok(Self {
                    #(#field_assignments,)*
                })
            }
        }
    }
}
