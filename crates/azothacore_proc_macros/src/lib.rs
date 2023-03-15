#![feature(proc_macro_span)]
#![feature(option_result_contains)]

extern crate proc_macro;

use itertools::Itertools;
use proc_macro::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Ident};

#[proc_macro]
pub fn scripts_registration(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Ident);
    let child_dir = input.to_string();

    let span = Span::call_site();
    let source = span.source_file();
    if !source.is_real() {
        // Generate nothing if the source isnt real, somehow
        return String::new().parse().unwrap();
    }
    // The directory where the source code resides. We wanna
    // generate the init() for that directory, one layer by layer
    let binding = source.path();
    let mut directory = binding.parent().unwrap().to_path_buf();
    directory.push(child_dir);
    if !directory.is_dir() {
        // Generate nothing if the source isnt real, somehow
        return String::new().parse().unwrap();
    }

    let script_names: Vec<Ident> = directory
        .read_dir()
        .expect("read directory failed in proc_macro!")
        .filter_map(|r| {
            let de = match r {
                Err(_) => return None,
                Ok(v) => v,
            };

            let de = de.path();
            let res = if de.file_name()? == source.path().file_name()? {
                None
            } else if de.is_dir() {
                de.file_name()
            } else if "rs" == de.extension()? && de.file_stem()?.to_str()?.chars().all(|c| c != '.') {
                // Pick up only rust files in directory.
                de.file_stem()
            } else {
                None
            };
            match res?.to_os_string().into_string() {
                Err(_) => None,
                Ok(v) => Some(format_ident!("{}", v)),
            }
        })
        .unique()
        .collect();

    let num_scripts = syn::Index::from(script_names.len());
    let modules = quote! {
        #(
            mod #script_names;
        )*
    };
    let register_fn = quote! {
        #[doc = "Runs through a run of init functions, returning early if at the first script that fails to register"]
        pub fn register() -> Result<(), Box<dyn std::error::Error>> {
            #(
                #script_names::init()?;
            )*
            Ok(())
        }
    };
    let registed_module_slice = quote! {
        pub const REGISTERED_MODULES: [&str; #num_scripts] = [
            #(
                stringify!(#script_names),
            )*
        ];
    };

    let expanded = quote! {
        #modules

        #register_fn

        #registed_module_slice
    };
    eprintln!("BODY>>>\n{}\n<<<BODY", expanded);

    TokenStream::from(expanded)
}
