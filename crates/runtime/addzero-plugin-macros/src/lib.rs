use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn addzero_plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn addzero_page(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn addzero_starter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let ident = function.sig.ident.clone();

    quote! {
        #function

        ::addzero_plugin_registry::inventory::submit! {
            ::addzero_plugin_registry::StarterRegistration {
                constructor: #ident,
            }
        }
    }
    .into()
}
