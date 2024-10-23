use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    io,
    mem,
    str::FromStr,
};

use flagset::{flags, FlagSet};
use num::Num;
use thiserror::Error;
pub mod wdc1;

pub trait DB2: Default {
    fn id(&self) -> u32;
    fn db2_file() -> &'static str;
    // TODO: break up the SQL stmt into db2_sql_locale_stmt
    fn db2_sql_stmt() -> &'static str;
    fn db2_sql_locale_stmt() -> Option<&'static str>;
    fn layout_hash() -> u32;
    /// id_index returns Some(x) if the WDC1 trait has an inlined ID index. otherwise return None (i.e. field count doesnt count)
    ///
    /// None is similar to `!loadInfo->Meta->HasIndexFieldInData()` in TrinityCore
    ///
    /// Some(x) is similar to `loadInfo->Meta->HasIndexFieldInData()` in TrinityCore
    fn inlined_id_index() -> Option<usize>;
    fn inline_parent_index() -> Option<usize>;
    fn num_fields() -> usize;
    /// Returns None if the WDC1 has a inlined parent id index.
    /// Else returns Some(x) if WDC1 has parent ID not in record
    /// i.e. parent ID is not in record
    fn non_inline_parent_index_type() -> Option<DB2FieldType>;
    /// Return a list of field types and their respective arities
    /// The indices of these fields should correspond to their field index
    fn db2_fields() -> BTreeMap<usize, (String, DB2FieldType, usize)>;
    /// WriteRecordData in TC
    fn to_raw_record_data(&self, locale: Locale) -> Vec<u8>;
    /// merge strings from ther raw record into the current record.
    fn merge_strs(&mut self, raw: &DB2RawRecord);
}

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
    LocalisedString,
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
            LocalisedString => None,
            String => None,
        }
    }
}

flags! {
    #[allow(non_camel_case_types)]
    #[derive(PartialOrd, Ord, Default, serde::Serialize, serde::Deserialize)]
    pub enum Locale: u32 {
        #[default]
        enUS = 1 << 0,
        koKR = 1 << 1,
        frFR = 1 << 2,
        deDE = 1 << 3,
        zhCN = 1 << 4,
        zhTW = 1 << 5,
        esES = 1 << 6,
        esMX = 1 << 7,
        ruRU = 1 << 8,
        none = 1 << 9,
        ptBR = 1 << 10,
        itIT = 1 << 11,
    }
}

impl Locale {
    pub fn to_flagset(self) -> FlagSet<Locale> {
        self.into()
    }

    pub fn to_num<N: Num>(&self) -> N {
        let mut bits = self.to_flagset().bits();
        let mut res = N::zero();
        while bits > 1 {
            res = res + N::one();
            bits >>= 1;
        }
        res
    }
}

impl TryFrom<u32> for Locale {
    type Error = LocaleError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let Ok(fs) = FlagSet::new(1 << value) else {
            return Err(LocaleError::FromNum { got: value });
        };
        let mut v = None;
        for f in fs.into_iter() {
            if v.is_some() {
                return Err(LocaleError::FromNum { got: value });
            }
            v = Some(f);
        }
        let Some(v) = v else { return Err(LocaleError::FromNum { got: value }) };
        Ok(v)
    }
}

#[derive(Error, Debug, Clone)]
pub enum LocaleError {
    #[error("locale string error: got {got}")]
    FromString { got: String },
    #[error("locale num error: got {got}")]
    FromNum { got: u32 },
}

impl FromStr for Locale {
    type Err = LocaleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "enUS" => Ok(Locale::enUS),
            "koKR" => Ok(Locale::koKR),
            "frFR" => Ok(Locale::frFR),
            "deDE" => Ok(Locale::deDE),
            "zhCN" => Ok(Locale::zhCN),
            "zhTW" => Ok(Locale::zhTW),
            "esES" => Ok(Locale::esES),
            "esMX" => Ok(Locale::esMX),
            "ruRU" => Ok(Locale::ruRU),
            "none" => Ok(Locale::none),
            "ptBR" => Ok(Locale::ptBR),
            "itIT" => Ok(Locale::itIT),
            _ => Err(LocaleError::FromString { got: s.to_string() }),
        }
    }
}

impl Locale {
    /// GetLocaleByName
    pub fn from_name(name: &str) -> Self {
        // including enGB case
        Self::from_str(name).unwrap_or(Locale::enUS)
    }
}

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, Default)]
pub struct LocalisedString {
    strings:        BTreeMap<Locale, String>,
    default_locale: Option<Locale>,
}

impl<'de> serde::Deserialize<'de> for LocalisedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let strings = BTreeMap::deserialize(deserializer)?;
        Ok(Self { strings, ..Default::default() })
    }
}

impl LocalisedString {
    pub fn set_by_locale(&mut self, locale: Locale, str: &str) -> io::Result<()> {
        if self.default_locale.is_none() {
            self.default_locale = Some(locale)
        }
        self.strings.insert(locale, str.to_string());
        Ok(())
    }

    /// merges localised strings available
    pub fn merge(&mut self, other: &Self) {
        for (l, s) in other.strings.iter() {
            self.strings.insert(*l, s.clone());
        }
    }

    pub fn str(&self, locale: Locale) -> &str {
        if let Some(s) = self.strings.get(&locale) {
            return s;
        }
        let locale = self.default_locale.unwrap_or_default();
        if let Some(s) = self.strings.get(&locale) {
            return s;
        }
        ""
    }
}

impl Display for LocalisedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.str(self.default_locale.unwrap_or_default()))
    }
}

pub fn new_localised_string() -> LocalisedString {
    LocalisedString {
        default_locale: None,
        strings:        BTreeMap::from_iter([
            (Locale::enUS, String::from("")),
            (Locale::koKR, String::from("")),
            (Locale::frFR, String::from("")),
            (Locale::deDE, String::from("")),
            (Locale::zhCN, String::from("")),
            (Locale::zhTW, String::from("")),
            (Locale::esES, String::from("")),
            (Locale::esMX, String::from("")),
            (Locale::ruRU, String::from("")),
            (Locale::none, String::from("")),
            (Locale::ptBR, String::from("")),
            (Locale::itIT, String::from("")),
        ]),
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
    LocalisedString(Vec<LocalisedString>),
    String(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct DB2RawRecord {
    pub id:     u32,
    pub fields: BTreeMap<usize, (String, DB2Field)>,
    pub parent: Option<DB2Field>,
}

pub fn raw_localised_strs_record_from_sql_row<'a, R: sqlx::Row>(
    db2_fields: &'a BTreeMap<usize, (String, DB2FieldType, usize)>,
    row: &'a R,
) -> sqlx::Result<DB2RawRecord>
where
    &'a str: sqlx::ColumnIndex<R>,
    u32: ::sqlx::decode::Decode<'a, R::Database>,
    u32: ::sqlx::types::Type<R::Database>,
    sqlx::types::Json<LocalisedString>: sqlx::decode::Decode<'a, R::Database>,
    sqlx::types::Json<LocalisedString>: sqlx::types::Type<R::Database>,
    sqlx::types::Json<Vec<LocalisedString>>: sqlx::decode::Decode<'a, R::Database>,
    sqlx::types::Json<Vec<LocalisedString>>: sqlx::types::Type<R::Database>,
{
    let mut record = DB2RawRecord {
        id:     row.try_get("id")?,
        fields: BTreeMap::new(),
        parent: None,
    };
    for (fi, (name, _, arity)) in db2_fields.iter().filter(|(_, (_, typ, _))| matches!(typ, DB2FieldType::LocalisedString)) {
        let res = if *arity <= 1 {
            let s: LocalisedString = row.try_get::<::sqlx::types::Json<_>, _>(name.as_str()).map(|x| x.0)?;
            vec![s]
        } else {
            let s: Vec<LocalisedString> = row.try_get::<::sqlx::types::Json<_>, _>(name.as_str()).map(|x| x.0)?;
            s
        };
        record.fields.insert(*fi, (name.clone(), DB2Field::LocalisedString(res)));
    }
    Ok(record)
}
