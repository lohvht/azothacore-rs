use std::io;

use byteorder::{LittleEndian, ReadBytesExt};

use super::basic_extractor::FileChunk;

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
