use std::{fs, path::Path, process};

use crate::AzResult;

/// create PID file
pub fn create_pid_file<P: AsRef<Path>>(filename: P) -> AzResult<u32> {
    let pid = process::id();
    fs::write(filename, pid.to_string().as_bytes())?;
    Ok(pid)
}
