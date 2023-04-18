//! Macros for botrs

mod describe;
mod proto_tool_describe;

use darling_macro::FromField;
use proc_macro::TokenStream;
use syn::DeriveInput;

#[derive(Clone, Debug, FromField)]
#[darling(forward_attrs(doc))]
struct DocumentedStructField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    // vis: syn::Visibility,
    attrs: Vec<syn::Attribute>,
}

/// The entry point for the `Describe` derive macro.
#[proc_macro_derive(Describe)]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    describe::expand_derive(&input)
}

/// The entry point for the `ProtoToolDescribe` derive macro.
#[proc_macro_derive(ProtoToolDescribe, attributes(tool))]
pub fn derive_proto_tool_describe(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    proto_tool_describe::expand_derive(&input)
}
