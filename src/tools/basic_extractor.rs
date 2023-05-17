mod wow7_3_5_26972;
use std::{
    io::{self, Read},
    marker::PhantomData,
    mem,
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use ordered_multimap::ListOrderedMultimap;
pub use wow7_3_5_26972::*;

use crate::{
    tools::extractor_common::casc_handles::{CascLocale, CascStorageHandle},
    GenericResult,
};

#[derive(Clone)]
pub struct FileChunk {
    pub fcc:        [u8; 4],
    pub size:       u32,
    pub data:       Vec<u8>,
    /// Sub-chunks. If the data contains chunks, sub_chunks will be populated as well.
    pub sub_chunks: ListOrderedMultimap<[u8; 4], FileChunk>,
    /// Makes it impossible to construct this manually
    phantom:        PhantomData<()>,
}

impl FileChunk {
    fn build(fcc: [u8; 4], size: u32, data: Vec<u8>) -> GenericResult<Self> {
        let mut s = Self {
            fcc,
            size,
            data,
            sub_chunks: ListOrderedMultimap::new(),
            phantom: PhantomData,
        };

        let mut ptr = io::Cursor::new(&s.data);
        let mut remaining_data = s.data.len();
        while remaining_data > 0 {
            let mut fcc = [0u8; 4];
            let mut size_to_subtract = ptr.read(&mut fcc[..])?;
            if size_to_subtract < fcc.len() {
                remaining_data -= size_to_subtract;
                continue;
            };
            fcc.reverse();
            if !INTERESTING_CHUNKS.iter().any(|e| *e == &fcc) {
                remaining_data -= size_to_subtract;
                continue;
            };
            let size = ptr.read_u32::<LittleEndian>()?;
            let mut data = vec![0u8; size as usize];
            ptr.read_exact(&mut data[..])?;

            size_to_subtract += mem::size_of_val(&size) + size as usize;
            remaining_data -= size_to_subtract;
            s.sub_chunks.append(fcc, FileChunk::build(fcc, size, data)?);
        }
        Ok(s)
    }
}

pub struct ChunkedFile {
    pub chunks: ListOrderedMultimap<[u8; 4], FileChunk>,
}

const INTERESTING_CHUNKS: [&[u8; 4]; 8] = [b"MVER", b"MAIN", b"MH2O", b"MCNK", b"MCVT", b"MWMO", b"MCLQ", b"MFBO"];

impl ChunkedFile {
    pub fn build<P>(storage: &CascStorageHandle, filename: P) -> GenericResult<Self>
    where
        P: AsRef<Path>,
    {
        let mut file = storage.open_file(filename, CascLocale::All.into())?;
        let file_size = file.get_file_size()? as usize;

        let mut buf: Vec<u8> = vec![];
        let mut read_file_size = file.read_to_end(&mut buf)?;

        if file_size != read_file_size {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("Unexpected file sizes while reading chunked file. expect {file_size}, got {read_file_size}"),
            )));
        }
        let mut ptr = io::Cursor::new(buf);

        let mut s = Self {
            chunks: ListOrderedMultimap::new(),
        };
        while read_file_size > 0 {
            let mut fcc = [0u8; 4];
            ptr.read_exact(&mut fcc[..])?;
            fcc.reverse();
            let mut size_to_subtract = fcc.len();
            let size = ptr.read_u32::<LittleEndian>()?;
            let mut data = vec![0u8; size as usize];
            ptr.read_exact(&mut data[..])?;

            size_to_subtract += mem::size_of_val(&size) + size as usize;
            read_file_size -= size_to_subtract;
            s.chunks.append(fcc, FileChunk::build(fcc, size, data)?);
        }

        Ok(s)
    }
}
