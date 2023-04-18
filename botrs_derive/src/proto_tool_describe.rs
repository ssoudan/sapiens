use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};

/// A derive macro for the `ProtoToolDescribe` trait.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(tool), supports(struct_named), forward_attrs(doc))]
struct DeriveReceiver {
    ident: syn::Ident,
    attrs: Vec<syn::Attribute>,
    generics: syn::Generics,

    name: Option<String>,

    input: syn::Path,
    output: syn::Path,
}

impl ToTokens for DeriveReceiver {
    fn to_tokens(&self, out: &mut proc_macro2::TokenStream) {
        let DeriveReceiver {
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
            panic!("Expected struct to have a doc string")
        } else {
            doc
        };

        let name = if let Some(name) = name {
            name.clone()
        } else {
            ident.to_string()
        };

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
                        description_context: "Use this when it is the best tool for the job.".to_string(), 
                        input_format: #input_ty::describe(),
                        output_format: #output_ty::describe(),
                    }
                }
            }
        })
    }
}

/// The entry point for the `ProtoToolDescribe` derive macro expansion.
pub fn expand_derive(input: &syn::DeriveInput) -> TokenStream {
    let receiver = match DeriveReceiver::from_derive_input(input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors().into(),
    };

    let tokens = quote! { #receiver };
    tokens.into()
}
