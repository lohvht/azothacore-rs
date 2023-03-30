use std::{fs, io, path::Path, process};

use flagset::InvalidBits;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("GenericError: {msg}")]
pub struct GenericError {
    pub msg: String,
}

#[derive(Error, Debug)]
#[error("invalid bits: {err}")]
pub struct InvalidBitsError {
    pub err: InvalidBits,
}

#[derive(Error, Debug)]
#[error("cannot create PID file {pid} (possible error: permission)")]
pub struct PIDFileError {
    pid:   u32,
    #[source]
    inner: io::Error,
}

/// create PID file
pub fn create_pid_file<P: AsRef<Path>>(filename: P) -> Result<u32, PIDFileError> {
    let pid = process::id();
    if let Err(e) = fs::write(filename, pid.to_string().as_bytes()) {
        return Err(PIDFileError { inner: e, pid });
    }
    Ok(pid)
}
