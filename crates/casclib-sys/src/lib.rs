// Suppress the flurry of warnings caused by using "C" naming conventions
#![expect(non_snake_case, non_camel_case_types, non_upper_case_globals, improper_ctypes, clippy::all)]
// Generated code calls unsafe fns without inner `unsafe {}` blocks, which edition 2024 warns about
#![allow(unsafe_op_in_unsafe_fn)]
// This matches bindgen::Builder output
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
