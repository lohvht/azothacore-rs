use std::{env, fs, path::PathBuf};

use proc_macro2::Span;
use quote::quote;
use syn::LitStr;

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=build.rs");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("build_compile_options.rs");

    let cargo_path = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = std::path::Path::new(std::str::from_utf8(&cargo_path).unwrap().trim());
    let cargo_workspace_dir = LitStr::new(&format!("{}", cargo_path.parent().unwrap().display()), Span::call_site());
    let tokens = quote! {
        const CARGO_WORKSPACE_DIR: &str = #cargo_workspace_dir;
    };

    fs::write(out_path, tokens.to_string()).unwrap();
}
