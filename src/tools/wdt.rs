use std::{collections::HashMap, io, path::Path};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{
    tools::{
        adt::AdtChunkModf,
        extractor_common::{casc_handles::CascStorageHandle, cstr_bytes_to_string, ChunkedFile, FileChunk},
    },
    GenericResult,
};

pub const WDT_MAP_SIZE: usize = 64;

#[derive(Default, Clone, Copy)]
pub struct WdtChunkMainSMAreaInfo {
    pub flag:  u32,
    pub data1: u32,
}

pub struct WdtChunkMain {
    pub fcc:      [u8; 4],
    pub size:     u32,
    pub adt_list: [[WdtChunkMainSMAreaInfo; WDT_MAP_SIZE]; WDT_MAP_SIZE],
}

impl From<FileChunk> for WdtChunkMain {
    fn from(value: FileChunk) -> Self {
        if value.fcc != *b"MAIN" {
            panic!("value.fcc must be MAIN, got {}", std::str::from_utf8(&value.fcc[..]).unwrap());
        }
        let d = WdtChunkMainSMAreaInfo::default();
        let mut cursor = io::Cursor::new(value.data);
        let mut adt_list = [[d; WDT_MAP_SIZE]; WDT_MAP_SIZE];
        for (_y, row) in adt_list.iter_mut().enumerate() {
            for (_x, col_val) in row.iter_mut().enumerate() {
                col_val.flag = cursor.read_u32::<LittleEndian>().unwrap();
                col_val.data1 = cursor.read_u32::<LittleEndian>().unwrap();
            }
        }

        Self {
            fcc: value.fcc,
            size: value.size,
            adt_list,
        }
    }
}

pub struct WDTFile {
    pub wmo_paths: HashMap<usize, String>,
    pub modf:      Option<AdtChunkModf>,
}

impl WDTFile {
    pub fn build<P: AsRef<Path>>(storage: &CascStorageHandle, storage_path: P) -> GenericResult<Self> {
        let file = ChunkedFile::build(storage, &storage_path)?;
        // .inspect_err(|e| {
        //     error!("Error opening wdt file at {}, err was {e}", storage_path.as_ref().display());
        // })?;

        let mut wmo_paths = HashMap::new();
        let mut modf = None;

        for (fourcc, chunk) in file.chunks {
            match &fourcc {
                b"MAIN" => {},
                b"MWMO" => {
                    let mut offset = 0;
                    let paths = chunk
                        .data
                        .split_inclusive(|b| *b == 0)
                        .map(|raw| {
                            // We dont anticipate a panic here as the strings will always be nul-terminated
                            let s = cstr_bytes_to_string(raw).unwrap();
                            let r = (offset, s);
                            offset += 1; // raw.len();
                            r
                        })
                        .collect::<HashMap<_, _>>();
                    wmo_paths.extend(paths);
                },
                b"MODF" => {
                    // global wmo instance data
                    modf = Some(AdtChunkModf::from(chunk));
                },
                _ => {},
            }
        }
        Ok(Self { wmo_paths, modf })
    }
}
