#![feature(const_option_ext)]
#![feature(fs_try_exists)]
#![feature(associated_type_bounds)]
#![feature(btree_extract_if)]
#![feature(result_option_inspect)]
#![feature(extract_if)]
#![feature(async_closure)]
#![feature(impl_trait_in_fn_trait_return)]
#![feature(async_fn_in_trait)]
#![feature(cursor_remaining)]

pub mod common;
pub mod compile_options;
pub mod logging;
pub mod macros;
pub mod modules;
pub mod server;
pub mod tools;

pub use compile_options::*;

pub type GenericResult<T> = Result<T, Box<dyn std::error::Error>>;
