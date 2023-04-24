use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Path, PathSegment};

use crate::DocumentedStructField;

/// A derive macro for the `Describe` trait.
#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct DeriveReceiver {
    ident: syn::Ident,
    data: darling::ast::Data<(), DocumentedStructField>,

    generics: syn::Generics,
}

fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
    match *ty {
        syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
        _ => None,
    }
}

fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
    let idents_of_path = path.segments.iter().fold(String::new(), |mut acc, v| {
        acc.push_str(&v.ident.to_string());
        acc.push('|');
        acc
    });
    vec!["Option|", "std|option|Option|", "core|option|Option|"]
        .into_iter()
        .find(|s| idents_of_path == *s)
        .and_then(|_| path.segments.last())
}

impl ToTokens for DeriveReceiver {
    fn to_tokens(&self, out: &mut proc_macro2::TokenStream) {
        let DeriveReceiver {
            ref ident,
            ref generics,
            ref data,
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();

        let fields = data.as_ref().take_struct().unwrap().fields;

        let doc_tuples = fields
            .into_iter()
            .map(|field| {
                let DocumentedStructField {
                    ref ident,
                    ref ty,
                    // ref vis,
                    ref attrs,
                    ..
                } = *field;

                let doc = attrs
                    .iter()
                    .filter(|attr| attr.path.is_ident("doc"))
                    .map(|attr| attr.parse_meta().unwrap())
                    .map(|meta| match meta {
                        syn::Meta::NameValue(syn::MetaNameValue {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }) => lit_str.value(),
                        _ => panic!("Expected doc attribute to be a string"),
                    })
                    .fold(String::new(), |mut acc, s| {
                        if !acc.is_empty() {
                            acc.push('\n');
                        }
                        acc.push_str(s.trim());
                        acc
                    });

                let doc = if doc.is_empty() {
                    "undocumented".to_string()
                } else {
                    doc
                };

                let is_optional = extract_type_path(ty)
                    .and_then(extract_option_segment)
                    .is_some();

                let ty = ty.to_token_stream().to_string();

                // Python-ify the type
                let ty = pythonify(ty);

                // add type information to the docstring
                // let doc = format!("<{}> {}", ty, doc);

                quote! {
                    sapiens::tools::FieldFormat{
                        name: stringify!(#ident).to_string(),
                        r#type: #ty.to_string(),
                        optional: #is_optional,
                        description: #doc.to_string(),
                    }
                }
            })
            .collect::<Vec<_>>();

        // dbg!(fields);
        out.extend(quote! {
            impl #imp Describe for #ident #ty #wher {
                fn describe() -> sapiens::tools::Format {
                    vec![
                         #(#doc_tuples),*
                    ].into()
                }
            }
        })
    }
}

fn pythonify(ty: String) -> String {
    ty.replace(' ', "")
        .replace("::", ".")
        .replace("Vec", "list")
        .replace("Option", "Optional")
        .replace("String", "str")
        .replace("i32", "int")
        .replace("i64", "int")
        .replace("f32", "float")
        .replace("f64", "float")
        .replace("Value", "Any")
        // .replace("bool", "bool")
        .replace("()", "None")
        .replace("HashMap", "dict")
        .replace('<', "[")
        .replace('>', "]")
}

/// The entry point for the `Describe` derive macro expansion.
pub fn expand_derive(input: &syn::DeriveInput) -> TokenStream {
    let receiver = match DeriveReceiver::from_derive_input(input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors().into(),
    };

    let tokens = quote! { #receiver };
    tokens.into()
}
