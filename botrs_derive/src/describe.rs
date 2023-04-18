use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};

#[derive(Clone, Debug, FromField)]
#[darling(forward_attrs(doc))]
struct StructField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    // vis: syn::Visibility,
    attrs: Vec<syn::Attribute>,
}

/// A derive macro for the `Describe` trait.
#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
pub struct DeriveReceiver {
    ident: syn::Ident,
    data: darling::ast::Data<(), StructField>,

    generics: syn::Generics,
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
                let StructField {
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
                            acc.push(' ');
                        }
                        acc.push_str(s.trim());
                        acc
                    });

                let doc = if doc.is_empty() {
                    "undocumented".to_string()
                } else {
                    doc
                };

                let ty = ty.to_token_stream().to_string();

                // Python-ify the type
                let ty = pythonify(ty);

                // add type information to the docstring
                let doc = format!("<{}> {}", ty, doc);

                quote! {
                    (stringify!(#ident).to_string(), #doc).into()
                }
            })
            .collect::<Vec<_>>();

        // dbg!(fields);
        out.extend(quote! {
            impl #imp Describe for #ident #ty #wher {
                fn describe() -> Format {
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
        // .replace("bool", "bool")
        .replace("()", "None")
        .replace("HashMap", "dict")
        .replace('<', "[")
        .replace('>', "]")
}

/// The entry point for the `Describe` derive macro expansion.
pub fn expand_derive_describe(input: &syn::DeriveInput) -> TokenStream {
    let receiver = match DeriveReceiver::from_derive_input(input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors().into(),
    };

    let tokens = quote! { #receiver };
    tokens.into()
}
