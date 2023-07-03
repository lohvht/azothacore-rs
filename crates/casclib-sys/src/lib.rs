// Suppress the flurry of warnings caused by using "C" naming conventions
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, improper_ctypes, clippy::all)]
// This matches bindgen::Builder output
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
