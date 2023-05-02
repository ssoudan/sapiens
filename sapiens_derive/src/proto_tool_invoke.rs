use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};

/// A derive macro for the `Describe` trait.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(tool_invoke_typed), supports(struct_named))]
pub struct DeriveReceiver {
    ident: syn::Ident,

    generics: syn::Generics,
    name: Option<syn::Path>,
}

impl ToTokens for DeriveReceiver {
    fn to_tokens(&self, out: &mut proc_macro2::TokenStream) {
        let DeriveReceiver {
            ref ident,
            ref generics,
            ref name,
            ..
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();

        let invoke_typed_name = name
            .clone()
            .unwrap_or_else(|| syn::parse_str("invoke_typed").unwrap());

        // dbg!(fields);
        out.extend(quote! {
            #[async_trait::async_trait]
            impl #imp ProtoToolInvoke for #ident #ty #wher {
                async fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
                    let input = serde_yaml::from_value(input).map_err(|e| ToolUseError::InvalidInput(e.to_string()))?;
                    let output = self.#invoke_typed_name(&input).await?;
                    Ok(serde_yaml::to_value(output).map_err(|e| ToolUseError::InvalidOutput(e.to_string()))?)
                }
            }
        })
    }
}

/// The entry point for the `ProtoToolInvoke` derive macro expansion.
pub fn expand_derive(input: &syn::DeriveInput) -> TokenStream {
    let receiver = match DeriveReceiver::from_derive_input(input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors().into(),
    };

    let tokens = quote! { #receiver };
    tokens.into()
}
