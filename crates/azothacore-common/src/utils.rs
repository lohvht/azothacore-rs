use std::{
    fs,
    io,
    net::{self, ToSocketAddrs},
    path::Path,
    process,
    sync::{Arc, OnceLock, Weak},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bincode::Options;

use crate::deref_boilerplate;

pub fn net_resolve<T: ToSocketAddrs>(t: T) -> io::Result<net::SocketAddr> {
    match t.to_socket_addrs()?.next() {
        None => Err(io::Error::new(io::ErrorKind::AddrNotAvailable, "Could not resolve address {addr_str}:{port}")),
        Some(a) => Ok(a),
    }
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

pub fn bincode_deserialise<R: io::Read, T: serde::de::DeserializeOwned>(r: &mut R) -> bincode::Result<T> {
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

pub fn unix_now() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

/// SharedFromSelfBase is the base implementation of C++'s std::shared_from_self
/// It contains a weak pointer to T.
pub struct SharedFromSelfBase<T> {
    weak: OnceLock<Weak<T>>,
}

impl<T> Default for SharedFromSelfBase<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SharedFromSelfBase<T> {
    pub const fn new() -> SharedFromSelfBase<T> {
        SharedFromSelfBase { weak: OnceLock::new() }
    }

    pub fn initialise(&self, r: &Arc<T>) {
        self.weak.get_or_init(|| Arc::downgrade(r));
    }
}

/// SharedFromSelf is the accompanying trait for C++'s std::shared_from_self
///
/// The old required method to be implemented is `get_base`.
///
/// Below is a contrived example of how to use this trait:
///
/// ```
/// use azothacore_common::utils::{SharedFromSelfBase, SharedFromSelf};
/// use std::sync::Arc;
///
/// struct MyStruct {
///     base: SharedFromSelfBase<MyStruct>,
/// }
/// impl SharedFromSelf<MyStruct> for MyStruct {
///     fn get_base(&self) -> &SharedFromSelfBase<MyStruct> {
///         &self.base
///     }
/// }
/// impl MyStruct {
///     fn new() -> Arc<MyStruct> {
///         let r = Arc::new(MyStruct {
///             base: SharedFromSelfBase::new(),
///         });
///         r.base.initialise(&r);
///         r
///     }
///     pub fn hello(&self) {
///         println!("Hello!");
///     }
/// }
/// let my_struct = MyStruct::new();
/// let my_struct_2 = my_struct.shared_from_self();
/// my_struct_2.hello();
/// ```
pub trait SharedFromSelf<T> {
    fn get_base(&self) -> &SharedFromSelfBase<T>;

    fn shared_from_self(&self) -> Arc<T> {
        self.get_base().weak.get().unwrap().upgrade().unwrap()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BufferDecodeError {
    #[error("insufficient bytes in buffer, have {have} bytes to read but want {wanted} bytes")]
    InsufficientBytes { have: usize, wanted: usize },
    #[error("unexpected bytes when attempting to decode: {0}")]
    UnexpectedDecode(String),
}

pub type BufferResult<D> = Result<D, BufferDecodeError>;

pub struct MessageBuffer {
    storage:       bytes::BytesMut,
    original_size: usize,
}

pub trait DecodeValueFromBytes {
    /// Decode should take the relevant data from the front of the given buffer
    /// and attempt decode the data.
    ///
    /// Once read successfully, the method should advance the buffer and return
    /// it back to a state that can be consumed by the next call. i.e. if decoding
    /// 2 bytes to form a u16, the method should also advance the buffer by 2 bytes
    /// and optionally run a [`bytes::BytesMut::reserve`] to perform any reallocations
    /// if needed.
    fn decode_from_bytes(buffer: &mut MessageBuffer) -> BufferResult<Self>
    where
        Self: std::marker::Sized;
}

impl DecodeValueFromBytes for u16 {
    fn decode_from_bytes(buffer: &mut MessageBuffer) -> BufferResult<Self> {
        let first = buffer.split_to(2);
        Ok(Self::from_be_bytes(first.to_vec().try_into().unwrap()))
    }
}

impl Default for MessageBuffer {
    fn default() -> Self {
        Self::new(4096)
    }
}

impl MessageBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            storage:       bytes::BytesMut::with_capacity(capacity),
            original_size: capacity,
        }
    }

    /// Attempt to reclaim the buffer that isn't being used
    pub fn normalise(&mut self) {
        let original = self.original_size;
        if self.capacity() <= original / 128 {
            self.reserve(original);
        }
    }

    pub fn read_value<V: DecodeValueFromBytes>(&mut self) -> BufferResult<V> {
        V::decode_from_bytes(self)
    }
}

deref_boilerplate!(MessageBuffer, bytes::BytesMut, storage);

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use super::MessageBuffer;

    #[test]
    fn message_buffer_read_value_rest() {
        let mut buf = MessageBuffer::default();
        buf.put_u16(1234);
        buf.extend(b"test string 1234");
        assert_eq!(buf.read_value::<u16>().unwrap(), 1234);
        assert_eq!(&buf[..], b"test string 1234");
    }
}
