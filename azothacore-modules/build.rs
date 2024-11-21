use std::{
    collections::BTreeMap,
    env,
    fs,
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};
use proc_macro2::Span;
use quote::{format_ident, quote};

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("build_modules_link.rs");
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .expect("crate directory should be able to be canonicalised");
    let dir_entries = fs::read_dir(crate_dir).expect("should not fail when reading directory entries from CARGO_MANIFEST_DIR");
    let mut module_name_to_path = BTreeMap::new();
    for entry in dir_entries {
        let entry = entry.expect("expect to be able to read directory entry");
        if entry.file_name() == "src" || entry.file_name() == "build.rs" {
            // treat `src` as dir + the actual source files for the crate.
            continue;
        }
        let epath = entry.path();
        if epath.extension().map_or(false, |ext| epath.is_file() && ext == "rs") {
            let mod_name = epath.file_stem().unwrap().to_string_lossy().to_string().to_case(Case::Snake);
            let mod_file_name = epath.file_name().unwrap().to_string_lossy().to_string();
            module_name_to_path.insert(mod_name, mod_file_name);
            continue;
        }
        if epath.is_dir() {
            let mod_name = epath.file_stem().unwrap().to_string_lossy().to_string();
            let mod_file_name = epath.join("mod.rs").to_str().unwrap().to_string();
            module_name_to_path.entry(mod_name.to_case(Case::Snake)).or_insert(mod_file_name);
            continue;
        }
    }
    let add_mod_systems = module_name_to_path.keys().map(|module_name| {
        let mn = format_ident!("{module_name}");
        quote! {
            _app.add_systems(bevy::prelude::Startup, #mn::init.in_set(crate::ModulesInitSet));
        }
    });
    let add_mod_systems_plugin = quote! {
        // Appends the list of modules to init as systems to [bevy::prelude::StartUp] stage,
        // Within each module the implementor is expected to add a register script function call.
        pub fn modules_plugin(_app: &mut bevy::prelude::App) {
            tracing::info!(modules=?MODULES_LIST, "initialising modules!");
            #(
                #add_mod_systems
            )*
        }
    };
    let module_strs = module_name_to_path.keys().map(|module_name| syn::LitStr::new(module_name, Span::call_site()));
    let modules_list = quote! {
        pub static MODULES_LIST: &[&str] = &[
            #(
                #module_strs,
            )*
        ];
    };

    let module_defs = module_name_to_path.into_iter().map(|(module_name, module_path)| {
        let mn = format_ident!("{module_name}");
        let mp = syn::LitStr::new(&module_path, Span::call_site());
        quote! {
            #[path = #mp]
            pub mod #mn;
        }
    });

    let tokens = quote! {
        #(
            #module_defs
        )*

        #add_mod_systems_plugin
        #modules_list
    };
    fs::write(out_path, prettyplease::unparse(&syn::parse_file(&tokens.to_string()).unwrap())).unwrap();
}
