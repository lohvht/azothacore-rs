use std::{collections::HashSet, env, path::PathBuf};

use cmake::Config;

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

// Example custom build script.
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");

    setup_casc_build();
}

fn setup_casc_build() {
    // Run cmake to build lib
    let dst = Config::new("CascLib")
        .define("CASC_BUILD_SHARED_LIB", "ON")
        .define("CASC_BUILD_STATIC_LIB", "OFF")
        .uses_cxx11()
        .very_verbose(true)
        .build();

    let wrapper_header = "WrapperCascLib.hpp";

    // Check output of `cargo build --verbose`, should see something like:
    // -L native=/path/runng/target/debug/build/runng-sys-abc1234/out
    // That contains output from cmake
    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
    // Tell rustc to use CascLib dync library
    println!("cargo:rustc-link-lib=dylib=casc");
    println!("cargo:rerun-if-changed={}", wrapper_header);

    let bindings = bindgen::Builder::default()
        .header(wrapper_header)
        // // This is needed if use `#include <nng.h>` instead of `#include "path/nng.h"`
        // .clang_arg("-Icpp/CascLib/src")
        // .clang_arg("-x c++")
        // .clang_arg("-std=c++14")
        .parse_callbacks(Box::new(IgnoreMacros(
            vec![
                "FP_INFINITE".into(),
                "FP_NAN".into(),
                "FP_NORMAL".into(),
                "FP_SUBNORMAL".into(),
                "FP_ZERO".into(),
                "IPPORT_RESERVED".into(),
            ]
            .into_iter()
            .collect(),
        )))
        .layout_tests(false)
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings");
}
