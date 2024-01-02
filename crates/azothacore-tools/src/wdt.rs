use std::{io, path::Path};

use azothacore_common::AzResult;
use byteorder::{LittleEndian, ReadBytesExt};

use crate::{
    adt::AdtChunkModf,
    extractor_common::{casc_handles::CascStorageHandle, cstr_bytes_to_string, ChunkedFile},
};

pub const WDT_MAP_SIZE: usize = 64;

#[derive(Default, Clone, Copy)]
pub struct WdtChunkMainSMAreaInfo {
    pub flag:  u32,
    pub data1: u32,
}

pub struct WdtChunkMain {
    pub adt_list: [[WdtChunkMainSMAreaInfo; WDT_MAP_SIZE]; WDT_MAP_SIZE],
}

impl From<(&[u8; 4], &[u8])> for WdtChunkMain {
    fn from(value: (&[u8; 4], &[u8])) -> Self {
        let (fcc, data) = value;
        if fcc != b"MAIN" {
            panic!("fcc must be MAIN, got {}", std::str::from_utf8(&fcc[..]).unwrap());
        }
        let d = WdtChunkMainSMAreaInfo::default();
        let mut cursor = io::Cursor::new(data);
        let mut adt_list = [[d; WDT_MAP_SIZE]; WDT_MAP_SIZE];
        for row in adt_list.iter_mut() {
            for col_val in row.iter_mut() {
                col_val.flag = cursor.read_u32::<LittleEndian>().unwrap();
                col_val.data1 = cursor.read_u32::<LittleEndian>().unwrap();
            }
        }

        Self { adt_list }
    }
}

pub struct WDTFile {
    pub wmo_paths: Vec<String>,
    pub modf:      Vec<AdtChunkModf>,
}

impl WDTFile {
    pub fn build<P: AsRef<Path>>(storage: &CascStorageHandle, storage_path: P) -> AzResult<Self> {
        let file = ChunkedFile::build(storage, &storage_path)?;
        // .inspect_err(|e| {
        //     error!("Error opening wdt file at {}, err was {e}", storage_path.as_ref().display());
        // })?;

        let mut wmo_paths = Vec::new();
        let mut modf = vec![];

        for (fourcc, chunk) in file.chunks() {
            match fourcc {
                b"MAIN" => {},
                b"MWMO" => {
                    for raw in chunk.split_inclusive(|b| *b == 0) {
                        // We dont anticipate a panic here as the strings will always be nul-terminated
                        let s = cstr_bytes_to_string(raw).unwrap();
                        wmo_paths.push(s);
                    }
                },
                b"MODF" => {
                    // global wmo instance data
                    modf.push(AdtChunkModf::from((fourcc, chunk)));
                },
                _ => {},
            }
        }
        Ok(Self { wmo_paths, modf })
    }
}
