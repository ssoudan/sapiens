//! not much - to be removed
#![allow(clippy::needless_continue)]
use darling::{FromDeriveInput, FromField};
use quote::{quote, ToTokens};
use syn::{parse_str, Expr};

#[derive(Clone, Debug, FromField)]
#[darling(forward_attrs(doc))]
struct StructField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    // vis: syn::Visibility,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct Cat2 {
    ident: syn::Ident,
    data: darling::ast::Data<(), StructField>,

    generics: syn::Generics,
}

impl ToTokens for Cat2 {
    fn to_tokens(&self, out: &mut proc_macro2::TokenStream) {
        let Self {
            ref ident,
            ref generics,
            ref data,
        } = *self;
        // let tokens = quote! {};
        dbg!(&ident);

        let (imp, ty, wher) = generics.split_for_impl();

        let fields = data.as_ref().take_struct().unwrap().fields;

        let fields = fields
            .into_iter()
            .map(|field| {
                let StructField {
                    ref ident,
                    ref ty,
                    // ref vis,
                    ref attrs,
                    ..
                } = *field;

                // let ident = ident.unwrap();

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

                // dbg!(&doc);

                dbg!(ty);

                // (ident, format!("<{}> {}", #ty, #doc))
                quote! {
                    (stringify!(#ident).to_string(), "<" + stringigy!(#ty) + ">" + #doc).into()
                }
            })
            .collect::<Vec<_>>();

        // dbg!(fields);

        out.extend(quote! {
            impl #imp Cat2 for #ident #ty #wher {
                fn describe(&self) -> Vec<Format> {
                    vec![
                         #(#fields),*
                    ].into()
                }
            }
        });
    }
}

fn main() {
    let input = r#"#[derive(Cat2)]
pub struct Foo {
    /// Hello I'm bar
    /// And I'm a multiline doc
    #[my_trait(volume = "whisper")]
    bar: bool,
    
    /// Hello I'm baz
    baz: i64,
    
    foo: Vec<String>,
}"#;

    let parsed = parse_str(input).unwrap();
    let receiver = Cat2::from_derive_input(&parsed).unwrap();
    let tokens = quote!(#receiver);

    println!(
        r"
INPUT:

{input}

PARSED AS:

{receiver:?}

EMITS:

{tokens}
    "
    );
}
