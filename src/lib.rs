#![feature(async_fn_in_trait)]
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
    fs,
    io,
    num,
    path::Path,
};

use bincode::Options;
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
    #[error("DB Error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("DB Loader Error: {0}")]
    DbLoad(#[from] DatabaseLoaderError),
    #[error("World Error: {0}")]
    World(#[from] WorldError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode serialisation/deserialisation error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("casc_handler error: {0}")]
    CascHandler(#[from] CascHandlerError),
    #[error("error parsing Integer from string: {0}")]
    StrToIntParse(#[from] num::ParseIntError),
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
    #[error("tokio join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("VMAP FACTORY LOAD ERROR: {0}")]
    VmapFactory(#[from] VmapFactoryLoadError),
    #[error("Detour error: {0}")]
    Detour(#[from] DetourStatus),
    #[error("Invalid bits: {0}")]
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

macro_rules! bincode_cfg {
    () => {{
        bincode::DefaultOptions::new()
            .with_no_limit()
            .with_little_endian()
            .with_varint_encoding()
            .allow_trailing_bytes()
    }};
}

pub fn bincode_serialise<W: io::Write, T: ?Sized + serde::Serialize>(w: &mut W, t: &T) -> bincode::Result<()> {
    bincode_cfg!().serialize_into(w, t)
}

pub fn bincode_deserialise<R: io::Read, T: ?Sized + serde::de::DeserializeOwned>(r: &mut R) -> bincode::Result<T> {
    bincode_cfg!().deserialize_from(r)
}

/// Set big buffer for now.
const DEFAULT_BUFFER_SIZE: usize = 256 * 1024 * 1024; // i.e. 256 Mebibyte

pub fn buffered_file_open<P: AsRef<Path>>(p: P) -> io::Result<io::BufReader<fs::File>> {
    Ok(io::BufReader::with_capacity(DEFAULT_BUFFER_SIZE, fs::File::open(p)?))
}

pub fn buffered_file_create<P: AsRef<Path>>(p: P) -> io::Result<io::BufWriter<fs::File>> {
    Ok(io::BufWriter::with_capacity(DEFAULT_BUFFER_SIZE, fs::File::create(p)?))
}
