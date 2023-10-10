use std::{fs, io, path::Path, process};

use bincode::Options;

/// create PID file
pub fn create_pid_file<P: AsRef<Path>>(filename: P) -> io::Result<u32> {
    let pid = process::id();
    fs::write(filename, pid.to_string().as_bytes())?;
    Ok(pid)
}

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
