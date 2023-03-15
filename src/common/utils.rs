use std::{error, fmt, fs, io, path::Path, process};

#[derive(Debug)]
pub struct PIDFileError {
    pid:   u32,
    inner: io::Error,
}

impl fmt::Display for PIDFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The wrapped error contains additional information and is available
        // via the source() method.
        write!(f, "cannot create PID file {} (possible error: permission)", self.pid)
    }
}

impl error::Error for PIDFileError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // The cause is the underlying implementation error type. Is implicitly
        // cast to the trait object `&error::Error`. This works because the
        // underlying type already implements the `Error` trait.
        Some(&self.inner)
    }
}

/// create PID file
pub fn create_pid_file<P: AsRef<Path>>(filename: P) -> Result<u32, PIDFileError> {
    let pid = process::id();
    if let Err(e) = fs::write(filename, pid.to_string().as_bytes()) {
        return Err(PIDFileError { inner: e, pid });
    }
    Ok(pid)
}
