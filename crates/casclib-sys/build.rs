use std::{collections::HashSet, env, path::PathBuf};

use cmake::Config;
#[expect(
    clippy::single_component_path_imports,
    unused_imports,
    reason = r#"
import here just as a reminder that by including this flate2 library (with the given features),
it will also statically link zlib to the final casclib-sys library too.
"#
)]
use flate2;

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
        .define("CASC_BUILD_SHARED_LIB", "OFF")
        .define("CASC_BUILD_STATIC_LIB", "ON")
        .uses_cxx11()
        .very_verbose(true)
        .build();

    let wrapper_header = "WrapperCascLib.hpp";

    // Check output of `cargo build --verbose`, should see something like:
    // -L native=/path/runng/target/debug/build/runng-sys-abc1234/out
    // That contains output from cmake
    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
    // Tell rustc to use CascLib dync library
    // println!("cargo:rustc-link-lib=dylib=casc");
    println!("cargo:rustc-link-lib=static=casc");
    println!("cargo:rerun-if-changed={}", wrapper_header);
    println!("cargo:rustc-link-lib=dylib=stdc++");
    // // NOTE: Usually If system has zlib, we need to include this line
    // // But we have forked the underlying casclib to not use
    // // system zlib for now.
    // println!("cargo:rustc-link-lib=dylib=z");

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
        .blocklist_type("LPMONITORINFOEXA?W?")
        .blocklist_type("LPTOP_LEVEL_EXCEPTION_FILTER")
        .blocklist_type("MONITORINFOEXA?W?")
        .blocklist_type("PEXCEPTION_FILTER")
        .blocklist_type("PEXCEPTION_ROUTINE")
        .blocklist_type("PSLIST_HEADER")
        .blocklist_type("PTOP_LEVEL_EXCEPTION_FILTER")
        .blocklist_type("PVECTORED_EXCEPTION_HANDLER")
        .blocklist_type("_?L?P?CONTEXT")
        .blocklist_type("_?L?P?EXCEPTION_POINTERS")
        .blocklist_type("_?P?DISPATCHER_CONTEXT.*")
        .blocklist_type("_?P?EXCEPTION_REGISTRATION_RECORD")
        .blocklist_type("_?P?IMAGE_TLS_DIRECTORY.*")
        .blocklist_type("_?P?NT_TIB")
        .blocklist_type("tagMONITORINFOEXA")
        .blocklist_type("tagMONITORINFOEXW")
        .blocklist_function("AddVectoredContinueHandler")
        .blocklist_function("AddVectoredExceptionHandler")
        .blocklist_function("CopyContext")
        .blocklist_function("GetThreadContext")
        .blocklist_function("GetXStateFeaturesMask")
        .blocklist_function("InitializeContext")
        .blocklist_function("InitializeContext2")
        .blocklist_function("InitializeSListHead")
        .blocklist_function("InterlockedFlushSList")
        .blocklist_function("InterlockedPopEntrySList")
        .blocklist_function("InterlockedPushEntrySList")
        .blocklist_function("InterlockedPushListSListEx")
        .blocklist_function("LocateXStateFeature")
        .blocklist_function("QueryDepthSList")
        .blocklist_function("RaiseFailFastException")
        .blocklist_function("RtlCaptureContext")
        .blocklist_function("RtlCaptureContext2")
        .blocklist_function("RtlFirstEntrySList")
        .blocklist_function("RtlInitializeSListHead")
        .blocklist_function("RtlInterlockedFlushSList")
        .blocklist_function("RtlInterlockedPopEntrySList")
        .blocklist_function("RtlInterlockedPushEntrySList")
        .blocklist_function("RtlInterlockedPushListSListEx")
        .blocklist_function("RtlQueryDepthSList")
        .blocklist_function("RtlRestoreContext")
        .blocklist_function("RtlUnwindEx")
        .blocklist_function("RtlVirtualUnwind")
        .blocklist_function("SetThreadContext")
        .blocklist_function("SetUnhandledExceptionFilter")
        .blocklist_function("SetXStateFeaturesMask")
        .blocklist_function("UnhandledExceptionFilter")
        .blocklist_function("__C_specific_handler")
        .layout_tests(false)
        .formatter(bindgen::Formatter::Rustfmt)
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings");
}
