//! Macros for botrs

mod describe;

use proc_macro::TokenStream;
use syn::DeriveInput;

/// The entry point for the `Describe` derive macro.
#[proc_macro_derive(Describe)]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    describe::expand_derive_describe(&input)
}
