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
#![feature(lint_reasons)]

pub mod common;
pub mod compile_options;
pub mod logging;
pub mod macros;
pub mod modules;
pub mod server;
pub mod tools;

use std::{
    fmt::{self, Arguments},
    num,
};

use common::{collision::management::VmapFactoryLoadError, configuration::ConfigError};
pub use compile_options::*;
use flagset::InvalidBits;
use server::{
    database::database_loader_utils::DatabaseLoaderError,
    game::world::WorldError,
    shared::recastnavigation_handles::DetourStatus,
};
use thiserror::Error;
use tools::extractor_common::casc_handles::CascHandlerError;

#[derive(Error, Debug)]
pub enum AzothaError {
    #[error("DB Error")]
    Db(#[from] sqlx::Error),
    #[error("DB Loader Error")]
    DbLoad(#[from] DatabaseLoaderError),
    #[error("World Error")]
    World(#[from] WorldError),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("bincode serialisation/deserialisation error")]
    Bincode(#[from] bincode::Error),
    #[error("casc_handler error")]
    CascHandler(#[from] CascHandlerError),
    #[error("error parsing Integer from string")]
    StrToIntParse(#[from] num::ParseIntError),
    #[error("config error")]
    Config(#[from] ConfigError),
    #[error("tokio join error")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("VMAP FACTORY LOAD ERROR")]
    VmapFactory(#[from] VmapFactoryLoadError),
    #[error("Detour error")]
    Detour(#[from] DetourStatus),
    #[error("Invalid bits")]
    InvalidBits(InvalidBits),
    #[error("err: {0}")]
    General(String),
}

impl From<InvalidBits> for AzothaError {
    fn from(value: InvalidBits) -> Self {
        AzothaError::InvalidBits(value)
    }
}

impl From<String> for AzothaError {
    fn from(value: String) -> Self {
        AzothaError::General(value)
    }
}

impl From<&str> for AzothaError {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

pub fn format_err(args: Arguments) -> AzothaError {
    if let Some(message) = args.as_str() {
        AzothaError::General(message.into())
    } else {
        // anyhow!("interpolate {var}"), can downcast to String
        AzothaError::General(fmt::format(args))
    }
}

pub type AzResult<T> = Result<T, AzothaError>;
