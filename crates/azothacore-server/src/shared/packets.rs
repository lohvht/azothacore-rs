use azothacore_common::{AzError, az_error};
use bytes::{BufMut, Bytes};

#[derive(Debug, thiserror::Error)]
pub enum ByteBufferError {
    /// ByteBufferPositionException in TC
    #[error("ByteBufferPositionException: pos {pos}, size {size}, wanted {wanted}")]
    Position { pos: usize, size: usize, wanted: usize },
    #[error(transparent)]
    Other(#[from] AzError),
}

/// ByteBuffer in TC / AC
pub struct ByteBuffer {
    storage:   bytes::BytesMut,
    rpos:      usize,
    bitpos:    usize,
    curbitval: u8,
}

impl Default for ByteBuffer {
    fn default() -> Self {
        Self::with_capacity(4096)
    }
}

impl ByteBuffer {
    pub fn with_capacity(reserve: usize) -> Self {
        Self {
            storage:   bytes::BytesMut::with_capacity(reserve),
            rpos:      0,
            bitpos:    8,
            curbitval: 0,
        }
    }

    /// ByteBuffer::FlushBits in TC
    pub fn flush_bits(&mut self) {
        if self.bitpos == 8 {
            return;
        }

        self.bitpos = 8;
        self.append(self.curbitval);
        self.curbitval = 0;
    }

    /// ByteBuffer::WriteBit in TC
    /// TODO: hirogoro@29oct2026:
    /// See if this + flush_bit can be actually refactored into
    /// an Iterator. The prevailing pattern appears to always follow up
    /// a list of WriteBit operations with FlushBit
    pub fn write_bit(&mut self, bit: bool) {
        self.bitpos -= 1;
        if bit {
            self.curbitval |= 1 << self.bitpos;
        }
        if self.bitpos == 0 {
            self.bitpos = 8;
            self.append(self.curbitval);
            self.curbitval = 0;
        }
    }

    /// ByteBuffer::WriteBits in TC
    pub fn write_bits(&mut self, value: impl Into<u64>, bits: usize) {
        let value = value.into();
        for i in (0..bits).rev() {
            self.write_bit(((value >> i) & 1) == 1);
        }
    }

    /// ByteBuffer::WriteString in TC:
    ///     Method for writing strings that have their length sent
    ///     separately in packet
    ///     without null-terminating the string
    pub fn write_string(&mut self, v: &str) {
        self.append(v.as_bytes());
    }

    /// ByteBuffer &operator<<(const std::string &) in TC
    pub fn write_clike_string(&mut self, v: &str) {
        self.append(v.as_bytes());
        self.append(0u8);
    }

    /// ByteBuffer::ResetBitPos in TC
    pub fn reset_bit_pos(&mut self) {
        if self.bitpos > 7 {
            return;
        }
        self.bitpos = 8;
        self.curbitval = 0;
    }

    pub fn read_bytes_without_advance(&self, wanted: usize) -> Result<&[u8], ByteBufferError> {
        if self.rpos + wanted > self.storage.len() {
            return Err(ByteBufferError::Position {
                pos: self.rpos,
                size: self.storage.len(),
                wanted,
            });
        }
        Ok(&self.storage[self.rpos..self.rpos + wanted])
    }

    /// Equivalent ByteBuffer::read(uint8 *dest, size_t len)
    pub fn read_bytes(&mut self, dest: &mut [u8]) -> Result<(), ByteBufferError> {
        self.reset_bit_pos();
        let bs = self.read_bytes_without_advance(dest.len())?;
        dest.copy_from_slice(bs);
        self.rpos += dest.len();
        Ok(())
    }

    pub fn read_string_with_length(&mut self, size: usize) -> Result<String, ByteBufferError> {
        let mut buf = vec![0; size];
        self.read_bytes(&mut buf)?;
        Ok(String::from_utf8_lossy(buf.as_slice()).into())
    }

    // ByteBuffer::ReadBit in TC
    pub fn read_bit(&mut self) -> Result<bool, ByteBufferError> {
        self.bitpos += 1;
        if self.bitpos > 7 {
            self.curbitval = self.read()?;
            self.bitpos = 0;
        }
        let res = ((self.curbitval >> (7 - self.bitpos)) & 1) != 0;
        Ok(res)
    }

    // ByteBuffer::ReadBits in TC
    pub fn read_bits(&mut self, bits: usize) -> Result<usize, ByteBufferError> {
        let mut value = 0;
        for i in (0..bits).rev() {
            if self.read_bit()? {
                value |= 1 << i;
            }
        }
        Ok(value)
    }
}

impl From<&[u8]> for ByteBuffer {
    fn from(value: &[u8]) -> Self {
        let mut buf = ByteBuffer::with_capacity(value.len());
        buf.append(value);
        buf
    }
}

impl From<Bytes> for ByteBuffer {
    fn from(value: Bytes) -> Self {
        Self::from(value.as_ref())
    }
}

impl From<ByteBuffer> for Bytes {
    fn from(value: ByteBuffer) -> Self {
        value.storage.freeze()
    }
}

pub trait AppendBufferBytes<T> {
    /// Equivalent of the various ByteBuffer &operator<< operations
    fn append(&mut self, v: T);
}

impl AppendBufferBytes<&[u8]> for ByteBuffer {
    fn append(&mut self, v: &[u8]) {
        if v.is_empty() {
            return;
        }
        self.flush_bits();
        self.storage.put_slice(v);
    }
}

impl<const N: usize> AppendBufferBytes<[u8; N]> for ByteBuffer {
    fn append(&mut self, v: [u8; N]) {
        if v.is_empty() {
            return;
        }
        self.flush_bits();
        self.storage.put_slice(v.as_slice());
    }
}

macro_rules! impl_append_buffer_bytes {
    ( $( $typ:ty ),+ $(,)? ) => {
        $(
            impl AppendBufferBytes<$typ> for ByteBuffer {
                fn append(&mut self, v: $typ) {
                    self.append(v.to_le_bytes());
                }
            }
        )+
    };
}

impl_append_buffer_bytes!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64,);

pub trait ReadBufferBytes<T> {
    /// Equivalent of the various ByteBuffer &operator>> operations
    fn read(&mut self) -> Result<T, ByteBufferError>;
}

impl<const N: usize> ReadBufferBytes<[u8; N]> for ByteBuffer {
    fn read(&mut self) -> Result<[u8; N], ByteBufferError> {
        self.reset_bit_pos();
        let bs = self.read_bytes_without_advance(N)?;
        let mut res = [0u8; N];
        res.copy_from_slice(bs);
        self.rpos += N;
        Ok(res)
    }
}

macro_rules! impl_read_buffer_bytes {
    ( $( $typ:ty ),+ $(,)? ) => {
        $(
            impl ReadBufferBytes<$typ> for ByteBuffer {
                fn read(&mut self) -> Result<$typ, ByteBufferError> {
                    self.read().map(<$typ>::from_le_bytes)
                }
            }
        )+
    };
}

impl_read_buffer_bytes! {
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
}

impl ReadBufferBytes<f32> for ByteBuffer {
    fn read(&mut self) -> Result<f32, ByteBufferError> {
        let res = self.read().map(<f32>::from_le_bytes)?;
        if !res.is_finite() {
            return Err(ByteBufferError::Other(az_error!("f32 is not finite")));
        }
        Ok(res)
    }
}

impl ReadBufferBytes<f64> for ByteBuffer {
    fn read(&mut self) -> Result<f64, ByteBufferError> {
        let res = self.read().map(<f64>::from_le_bytes)?;
        if !res.is_finite() {
            return Err(ByteBufferError::Other(az_error!("f64 is not finite")));
        }
        Ok(res)
    }
}

impl ReadBufferBytes<String> for ByteBuffer {
    fn read(&mut self) -> Result<String, ByteBufferError> {
        let remaining = &self.storage[self.rpos..];
        let nul_idx = remaining.iter().position(|x| *x == 0);
        let read_past_nul_idx = nul_idx.is_some();
        let len = nul_idx.unwrap_or(remaining.len());
        let res: String = String::from_utf8_lossy(&remaining[..len]).into();
        if !remaining.is_empty() {
            // Resets the bit position only if there is something to be read
            // from the remaining buffer.
            // If nothing is read, a potential call to `read_bit` might still
            // continue fetching based on the same byte.
            self.reset_bit_pos();
        }
        self.rpos += len + if read_past_nul_idx { 1 } else { 0 };
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_produces_the_right_bits() {
        let mut bs = ByteBuffer::default();
        bs.write_bit(true);
        bs.write_bits(0b101u16, 3);
        bs.flush_bits();
        let res: u8 = bs.read().unwrap();
        assert_eq!(res, 0b1101_0000);
    }
}
