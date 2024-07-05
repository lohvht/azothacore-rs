use std::{collections::BTreeMap, mem};

pub mod wdc1;

#[derive(Debug)]
pub enum DB2FieldType {
    I64,
    I32,
    I16,
    I8,
    U64,
    U32,
    U16,
    U8,
    F32,
    String,
}

impl DB2FieldType {
    pub fn field_size(&self) -> Option<usize> {
        use DB2FieldType::*;
        match self {
            I64 => Some(mem::size_of::<i64>()),
            I32 => Some(mem::size_of::<i32>()),
            I16 => Some(mem::size_of::<i16>()),
            I8 => Some(mem::size_of::<i8>()),
            U64 => Some(mem::size_of::<u64>()),
            U32 => Some(mem::size_of::<u32>()),
            U16 => Some(mem::size_of::<u16>()),
            U8 => Some(mem::size_of::<u8>()),
            F32 => Some(mem::size_of::<f32>()),
            String => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LocalisedString {
    strings:        [String; 12],
    default_locale: Option<usize>,
}

impl LocalisedString {
    pub fn set_by_locale_as_num(&mut self, locale_as_number: usize, str: String) {
        if self.default_locale.is_none() {
            self.default_locale = Some(locale_as_number)
        }
        self.strings[locale_as_number] = str
    }

    pub fn def_str(&self) -> String {
        let idx = self.default_locale.unwrap_or_default();
        self.str(idx)
    }

    pub fn str(&self, locale_as_number: usize) -> String {
        self.strings[locale_as_number].clone()
    }
}

pub fn new_localised_string() -> LocalisedString {
    LocalisedString {
        default_locale: None,
        strings:        [
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
        ],
    }
}

#[derive(Debug, Clone)]
pub enum DB2Field {
    I64(Vec<i64>),
    I32(Vec<i32>),
    I16(Vec<i16>),
    I8(Vec<i8>),
    U64(Vec<u64>),
    U32(Vec<u32>),
    U16(Vec<u16>),
    U8(Vec<u8>),
    F32(Vec<f32>),
    String(Vec<LocalisedString>),
}

#[derive(Debug, Clone)]
pub struct DB2RawRecord {
    pub id:     u32,
    pub fields: BTreeMap<usize, DB2Field>,
    pub parent: Option<DB2Field>,
}
