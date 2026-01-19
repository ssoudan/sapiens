//! The `ProtoToolDescribe` derive macro.
#![allow(clippy::needless_continue)]
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::Expr;

/// A derive macro for the `ProtoToolDescribe` trait.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(tool), supports(struct_named), forward_attrs(doc))]
struct DeriveReceiver {
    /// The struct or enum that the derive macro is being applied to.
    ident: syn::Ident,
    /// The attributes of the struct or enum.
    attrs: Vec<syn::Attribute>,
    /// The generics of the struct or enum.
    generics: syn::Generics,
    /// The name
    name: Option<String>,
    /// The input type
    input: syn::Path,
    /// The output type
    output: syn::Path,
}

impl ToTokens for DeriveReceiver {
    fn to_tokens(&self, out: &mut proc_macro2::TokenStream) {
        let Self {
            ref ident,
            ref attrs,
            ref generics,
            ref name,
            ref input,
            ref output,
            ..
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();

        let doc = attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .filter_map(|attr| match &attr.meta {
                syn::Meta::NameValue(syn::MetaNameValue {
                    value:
                        Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }),
                    ..
                }) => Some(s.value()),
                _ => None,
            })
            .fold(String::new(), |mut acc, s| {
                if !acc.is_empty() {
                    acc.push('\n');
                }
                acc.push_str(s.trim());
                acc
            });

        let doc = if doc.is_empty() {
            panic!("Expected struct to have a doc string")
        } else {
            doc
        };

        let name = name
            .as_ref()
            .map_or_else(|| ident.to_string(), std::clone::Clone::clone);

        // dbg!(input);

        // dbg!(output);

        let input_ty = &input.segments.last().unwrap().ident;
        let output_ty = &output.segments.last().unwrap().ident;

        // dbg!(fields);
        out.extend(quote! {
            impl #imp ProtoToolDescribe for #ident #ty #wher {
                fn description(&self) -> ToolDescription {
                    ToolDescription {
                        name: #name.to_string(),
                        description: #doc.to_string(),
                        parameters: #input_ty::describe(),
                        responses_content: #output_ty::describe(),
                    }
                }
            }
        });
    }
}

/// The entry point for the `ProtoToolDescribe` derive macro expansion.
pub(crate) fn expand_derive(input: &syn::DeriveInput) -> TokenStream {
    let receiver = match DeriveReceiver::from_derive_input(input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors().into(),
    };

    let tokens = quote! { #receiver };
    tokens.into()
}
