mod derive_wdc1_impl;

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(WDC1, attributes(layout_hash, id, parent))]
pub fn derive_wdc1(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_wdc1_impl::derive(&input).unwrap_or_else(|err| err.to_compile_error()).into()
}
