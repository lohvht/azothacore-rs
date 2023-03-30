#![feature(const_option_ext)]
#![feature(fs_try_exists)]
#![feature(associated_type_bounds)]
#![feature(btree_drain_filter)]
#![feature(result_option_inspect)]
#![feature(drain_filter)]
#![feature(async_closure)]
#![feature(impl_trait_in_fn_trait_return)]
#![feature(async_fn_in_trait)]

pub mod common;
pub mod compile_options;
pub mod logging;
pub mod macros;
pub mod modules;
pub mod server;

pub use compile_options::*;

pub type GenericResult = Result<(), Box<dyn std::error::Error>>;
