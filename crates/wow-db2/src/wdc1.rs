use std::{
    collections::BTreeMap,
    ffi::CStr,
    io::{self, Read},
    marker::PhantomData,
    mem,
    vec,
};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use flagset::{flags, FlagSet};
use itertools::{FoldWhile, Itertools};

use crate::{new_localised_string, DB2Field, DB2FieldType, DB2RawRecord};

flags! {
    pub enum WDCFlags: u16 {
        HasOffsetMap = 0x01,
        HasRelationshipData = 0x02,
        /// If this flag exists, field count does not include ID, and ID is implicitly assumed to be first field
        HasNonInlinedIDs = 0x04,
        IsBitpacked = 0x10,
    }
}

#[derive(Default, Debug, Clone)]
pub struct WDC1Header {
    /// 'WDC1'
    /// Signature in TrinityCore
    magic: [u8; 4],
    /// RecordCount in TrinityCore
    pub record_count: u32,
    /// FieldCount in TrinityCore
    pub field_count: u32,
    /// RecordSize in TrinityCore
    pub record_size: u32,
    /// StringTableSize in TrinityCore
    pub string_table_size: u32,
    /// hash of the table name
    /// TableHash in TrinityCore
    pub table_hash: u32,
    /// this is a hash field that changes only when the structure of the data changes
    /// LayoutHash in TrinityCore
    pub layout_hash: u32,
    /// MinId in TrinityCore
    pub min_id: u32,
    /// MaxId in TrinityCore
    pub max_id: u32,
    /// as seen in TextWowEnum
    /// Locale in TrinityCore
    pub locale: u32,
    /// CopyTableSize in TrinityCore
    pub copy_table_size: u32,
    /// possible values are listed in Known Flag Meanings
    /// Flags in TrinityCore
    pub flags: FlagSet<WDCFlags>,
    /// this is the index of the field containing ID values, this is ignored if flags & 0x04 != 0
    /// IndexField in TrinityCore
    pub id_index: u16,
    /// from WDC1 onwards, this value seems to always be the same as the 'field_count' value
    /// TotalFieldCount in TrinityCore
    pub total_field_count: u32,
    /// relative position in record where bitpacked data begins, not important for parsing the file
    /// PackedDataOffset in TrinityCore
    pub bitpacked_data_offset: u32,
    /// ParentLookupCount in TrinityCore
    pub lookup_column_count: u32,
    /// Offset to array of struct { u32 offset, u16 size,}[max_id - min_id + 1],
    /// CatalogDataOffset in TrinityCore
    pub offset_map_offset: u32,
    /// List of ids present in the DB file
    /// IdTableSize in TrinityCore
    pub id_list_size: u32,
    /// ColumnMetaSize in TrinityCore
    pub field_storage_info_size: u32,
    /// CommonDataSize in TrinityCore
    pub common_data_size: u32,
    /// PalletDataSize in TrinityCore
    pub pallet_data_size: u32,
    /// ParentLookupDataSize in TrinityCore
    pub relationship_data_size: u32,
}

impl WDC1Header {
    fn flag_check(&self, fs: FlagSet<WDCFlags>) -> bool {
        (self.flags & fs) == fs
    }

    fn has_offset_map_flag(&self) -> bool {
        self.flag_check(WDCFlags::HasOffsetMap.into())
    }

    fn id_index_check<W>(&self) -> io::Result<()>
    where
        W: WDC1,
    {
        let (is_none, id_idx) = match W::id_index() {
            None => (true, 0),
            Some(e) => (false, e),
        };

        if self.flag_check(WDCFlags::HasNonInlinedIDs.into()) && is_none {
            return Ok(());
        }

        if self.id_index as usize == id_idx {
            return Ok(());
        }
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("id_index from code is not correct. id_index from dbc is {:?}, got {:?}", self.id_index, id_idx,),
        ))
    }

    fn magic_check(&self) -> io::Result<()> {
        if &self.magic != b"WDC1" {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("WRONG MAGIC CHECK: expect {:?} got {:?}", b"WDC1", self.magic),
            ));
        }
        Ok(())
    }

    fn layout_hash_check<W>(&self) -> io::Result<()>
    where
        W: WDC1,
    {
        if self.layout_hash != W::layout_hash() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("WRONG LAYOUT_HASH: expect {:?} got {:?}", W::layout_hash(), self.layout_hash),
            ));
        }
        Ok(())
    }

    fn init_from_reader<R>(&mut self, rdr: &mut R) -> io::Result<()>
    where
        R: io::Read,
    {
        rdr.read_exact(&mut self.magic[..])?;
        self.record_count = rdr.read_u32::<LittleEndian>()?;
        self.field_count = rdr.read_u32::<LittleEndian>()?;
        self.record_size = rdr.read_u32::<LittleEndian>()?;
        self.string_table_size = rdr.read_u32::<LittleEndian>()?;
        self.table_hash = rdr.read_u32::<LittleEndian>()?;
        self.layout_hash = rdr.read_u32::<LittleEndian>()?;
        self.min_id = rdr.read_u32::<LittleEndian>()?;
        self.max_id = rdr.read_u32::<LittleEndian>()?;
        self.locale = rdr.read_u32::<LittleEndian>()?;
        self.copy_table_size = rdr.read_u32::<LittleEndian>()?;

        let f = rdr.read_u16::<LittleEndian>()?;
        self.flags = FlagSet::<WDCFlags>::new(f).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("FLAGS INVALID?: got {:?}, err was {}", f, e)))?;
        self.id_index = rdr.read_u16::<LittleEndian>()?;
        self.total_field_count = rdr.read_u32::<LittleEndian>()?;
        self.bitpacked_data_offset = rdr.read_u32::<LittleEndian>()?;
        self.lookup_column_count = rdr.read_u32::<LittleEndian>()?;
        self.offset_map_offset = rdr.read_u32::<LittleEndian>()?;
        self.id_list_size = rdr.read_u32::<LittleEndian>()?;
        self.field_storage_info_size = rdr.read_u32::<LittleEndian>()?;
        self.common_data_size = rdr.read_u32::<LittleEndian>()?;
        self.pallet_data_size = rdr.read_u32::<LittleEndian>()?;
        self.relationship_data_size = rdr.read_u32::<LittleEndian>()?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct FieldStructure {
    /// size in bits as calculated by: byteSize = (32 - size) / 8;
    /// this value can be negative to indicate field sizes larger than 32-bits
    pub size:     i16,
    /// position of the field within the record, relative to the start of the record
    pub position: u16,
}

impl FieldStructure {
    fn init_from_reader<R>(rdr: &mut R, total_field_count: u32) -> io::Result<Vec<Self>>
    where
        R: io::Read,
    {
        let mut res = vec![];
        for _ in 0..total_field_count {
            res.push(Self {
                size:     rdr.read_i16::<LittleEndian>()?,
                position: rdr.read_u16::<LittleEndian>()?,
            })
        }
        Ok(res)
    }
}

#[derive(Debug)]
enum FieldCompression {
    /// None -- the field is a 8-, 16-, 32-, or 64-bit integer in the record data
    ///
    /// This is DB2ColumnCompression::None in TrinityCore
    None,
    /// Bitpacked -- the field is a bitpacked integer in the record data. It
    /// is `field_size_bits` long and starts at `field_offset_bits`.
    /// A bitpacked value occupies
    ///   (`field_size_bits` + (`field_offset_bits` & 7) + 7) / 8
    /// bytes starting at byte
    ///   `field_offset_bits` / 8
    /// in the record data.  These bytes should be read as a little-endian value,
    /// then the value is shifted to the right by (`field_offset_bits` & 7) and
    /// masked with ((1ull << `field_size_bits`) - 1).
    /// This is DB2ColumnCompression::Immediate in TrinityCore
    ///
    /// offset_bits - not useful for most purposes; formula they use to calculate is `offset_bits` = `field_offset_bits` - (`header.bitpacked_data_offset` * 8)
    /// BitOffset in TrinityCore
    ///
    /// size_bits -  not useful for most purposes;
    /// BitWidth in TrinityCore
    ///
    /// flags - known values - 0x01: sign-extend (signed)
    BitpackedInlined { offset_bits: u32, size_bits: u32, _flags: u32 },
    /// Common data -- the field is assumed to be a default value, and exceptions
    /// from that default value are stored in the corresponding section in
    /// common_data as pairs of { uint32_t record_id; uint32_t value; }.
    ///
    /// This is DB2ColumnCompression::CommonData in TrinityCore
    CommonData { default_value: u32 },
    /// Bitpacked indexed -- the field has a bitpacked index in the record data.
    /// This index is used as an index into the corresponding section in
    /// pallet_data.  The pallet_data section is an array of uint32_t, so the index
    /// should be multiplied by 4 to obtain a byte offset.
    ///
    /// offset_bits - not useful for most purposes; formula they use to calculate is `offset_bits` = field_offset_bits - (header.bitpacked_data_offset * 8)
    /// BitOffset in TrinityCore
    ///
    /// size_bits -  not useful for most purposes
    /// BitWidth in TrinityCore
    ///
    /// This is DB2ColumnCompression::Pallet in TrinityCore
    BitpackedIndexed { offset_bits: u32, size_bits: u32 },
    /// Bitpacked indexed array -- the field has a bitpacked index in the record
    /// data.  This index is used as an index into the corresponding section in
    /// pallet_data.  The pallet_data section is an array of uint32_t[array_count].
    ///
    /// offset_bits - not useful for most purposes; formula they use to calculate is `offset_bits` = field_offset_bits - (header.bitpacked_data_offset * 8)
    /// BitOffset in TrinityCore
    ///
    /// size_bits -  not useful for most purposes
    /// BitWidth in TrinityCore
    ///
    /// This is DB2ColumnCompression::PalletArray in TrinityCore
    BitpackedIndexedArray { offset_bits: u32, size_bits: u32, array_count: u32 },
}

impl FieldCompression {
    fn from_vals(comp_type_uint: u32, v1: u32, v2: u32, v3: u32) -> io::Result<Self> {
        Ok(match comp_type_uint {
            0 => FieldCompression::None,
            1 => FieldCompression::BitpackedInlined {
                offset_bits: v1,
                size_bits:   v2,
                _flags:      v3,
            },
            2 => FieldCompression::CommonData { default_value: v1 },
            3 => FieldCompression::BitpackedIndexed {
                offset_bits: v1,
                size_bits:   v2,
            },
            4 => FieldCompression::BitpackedIndexedArray {
                offset_bits: v1,
                size_bits:   v2,
                array_count: v3,
            },
            n => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("field compression values are invalid: expect 0-4 got {:?}", n),
                ))
            },
        })
    }
}

#[derive(Debug)]
struct FieldStorageRaw {
    field_offset_bits:    u16,
    field_size_bits:      u16,
    additional_data_size: u32,
    storage_type:         u32,
    value1:               u32,
    value2:               u32,
    value3:               u32,
}

impl FieldStorageRaw {
    fn init_from_reader<R>(rdr: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        Ok(Self {
            field_offset_bits:    rdr.read_u16::<LittleEndian>()?,
            field_size_bits:      rdr.read_u16::<LittleEndian>()?,
            additional_data_size: rdr.read_u32::<LittleEndian>()?,
            storage_type:         rdr.read_u32::<LittleEndian>()?,
            value1:               rdr.read_u32::<LittleEndian>()?,
            value2:               rdr.read_u32::<LittleEndian>()?,
            value3:               rdr.read_u32::<LittleEndian>()?,
        })
    }
}

/// Also known as DB2ColumnMeta in triinitycore
#[derive(Debug)]
struct FieldStorageInfo {
    field_offset_bits:    u16,
    /// very important for reading bitpacked fields; size is the sum of
    /// all array pieces in bits - for example, uint32[3] will appear here as '96'
    _field_size_bits:     u16,
    /// additional_data_size is the size in bytes of the corresponding section in
    /// common_data or pallet_data.  These sections are in the same order as the
    /// field_info, so to find the offset, add up the additional_data_size of any
    /// previous fields which are stored in the same block (common_data or
    /// pallet_data).
    /// AdditionalDataSize in trinitycore
    additional_data_size: u32,
    storage_type:         FieldCompression,
}

impl FieldStorageInfo {
    fn init_from_reader<R>(header: &WDC1Header, rdr: &mut R) -> io::Result<Vec<Self>>
    where
        R: io::Read,
    {
        let mut res = vec![];
        for _ in 0..(header.field_storage_info_size as usize / mem::size_of::<FieldStorageRaw>()) {
            let raw = FieldStorageRaw::init_from_reader(rdr)?;
            res.push(Self {
                additional_data_size: raw.additional_data_size,
                field_offset_bits:    raw.field_offset_bits,
                _field_size_bits:     raw.field_size_bits,
                storage_type:         FieldCompression::from_vals(raw.storage_type, raw.value1, raw.value2, raw.value3)?,
            })
        }
        Ok(res)
    }
}

#[derive(Debug)]
struct DB2RecordCopy {
    /// NewRowId in TrinityCore
    id_of_new_row:    u32,
    /// SourceRowId in TrinityCore
    id_of_copied_row: u32,
}

/// In some tables, this relationship mapping replaced columns that were used
/// only as a lookup, such as the SpellID in SpellX* tables.
struct RelationshipData {
    /// This is the id of the foreign key for the record, e.g. SpellID in
    /// SpellX* tables.
    foreign_id:   u32,
    /// This is the index of the record in record_data.  Note that this is
    /// *not* the record's own ID.
    record_index: u32,
}

struct RelationshipMapping {
    _num_entries: u32,
    _min_id:      u32,
    _max_id:      u32,
    entries:      Vec<RelationshipData>,
}

impl RelationshipMapping {
    fn init_from_reader<R>(header: &WDC1Header, rdr: &mut R) -> io::Result<Vec<Self>>
    where
        R: io::Read,
    {
        let mut res = vec![];
        for _ in 0..header.lookup_column_count {
            let _num_entries = rdr.read_u32::<LittleEndian>()?;
            let _min_id = rdr.read_u32::<LittleEndian>()?;
            let _max_id = rdr.read_u32::<LittleEndian>()?;

            let mut entries = vec![];
            for _ in 0.._num_entries {
                entries.push(RelationshipData {
                    foreign_id:   rdr.read_u32::<LittleEndian>()?,
                    record_index: rdr.read_u32::<LittleEndian>()?,
                })
            }
            res.push(Self {
                _num_entries,
                _min_id,
                _max_id,
                entries,
            })
        }
        Ok(res)
    }
}

pub trait WDC1: Default {
    fn layout_hash() -> u32;
    /// id_index returns Some(x) if the WDC1 trait has an inlined ID index. otherwise return None (i.e. field count doesnt count)
    ///
    /// None is similar to `!loadInfo->Meta->HasIndexFieldInData()` in TrinityCore
    ///
    /// Some(x) is similar to `loadInfo->Meta->HasIndexFieldInData()` in TrinityCore
    fn id_index() -> Option<usize>;
    fn num_fields() -> usize;
    /// Returns Some(x) if the WDC1 has a non inlined id index.
    fn non_inline_parent_index_type() -> Option<DB2FieldType>;
    /// Return a list of field types and their respective arities
    /// The indices of these fields should correspond to their field index
    fn db2_fields() -> BTreeMap<usize, (DB2FieldType, usize)>;
}

#[derive(Debug)]
struct OffsetMapEntry {
    offset: u32,
    size:   u16,
}

#[derive(Debug)]
enum FileLoaderData {
    Regular {
        regular_records_data: Vec<u8>,
        strings_table:        Vec<u8>,
    },
    OffsetMaps {
        variable_record_data:        Vec<u8>,
        // _catalogue in trinitycore
        offset_map:                  Vec<OffsetMapEntry>,
        number_of_catalogue_entries: usize,
    },
}

pub struct FileLoader<W>
where
    W: WDC1,
{
    locale:                u32,
    header:                WDC1Header,
    field_data:            Vec<FieldStructure>,
    id_list:               Vec<u32>,
    copy_table:            Vec<DB2RecordCopy>,
    field_storage_infos:   Vec<FieldStorageInfo>,
    /// Pallet data, contains a map of field indices to an array of 32bit blocks for that field
    pallet_data:           BTreeMap<usize, Vec<[u8; 4]>>,
    /// Pallet array data, contains a map of field indices to an array of array of 32bit blocks for that field
    pallet_array_data:     BTreeMap<usize, Vec<Vec<[u8; 4]>>>,
    /// Common data, contains a map of field indices to a map of record IDs to the actual 32bit blocks value
    common_data:           BTreeMap<usize, BTreeMap<u32, [u8; 4]>>,
    relationship_mappings: Vec<RelationshipMapping>,
    file_data:             FileLoaderData,
    data_start:            usize,
    wdc1:                  PhantomData<W>,
}

impl<W> FileLoader<W>
where
    W: WDC1 + From<DB2RawRecord>,
{
    pub fn from_reader<R>(mut rdr: R, locale: u32) -> Result<FileLoader<W>, io::Error>
    where
        R: io::Read,
    {
        let mut buf: Vec<u8> = vec![];
        let file_size = rdr.read_to_end(&mut buf)?;
        let mut buf = io::Cursor::new(buf);

        let mut header = WDC1Header::default();
        header.init_from_reader(&mut buf)?;
        header.magic_check()?;
        header.layout_hash_check::<W>()?;
        header.id_index_check::<W>()?;

        if !header.has_offset_map_flag() {
            let expected_file_size = mem::size_of::<WDC1Header>()
                + mem::size_of::<FieldStructure>() * header.field_count as usize
                + header.record_count as usize * header.record_size as usize
                + header.string_table_size as usize
                + header.id_list_size as usize
                + header.copy_table_size as usize
                + header.field_storage_info_size as usize
                + header.pallet_data_size as usize
                + header.common_data_size as usize
                + header.relationship_data_size as usize;

            if expected_file_size != file_size {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("UNEXPECTED FILE SIZE: expect {:?} got {:?}", expected_file_size, file_size),
                ));
            }
        }
        if (header.locale & (1 << locale)) == 0 {
            let header_locale = header.locale;
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Attempted to load locale {locale} for db2 which has locales {header_locale}. Check if you placed your localised db2 files in correct directory."),
            ));
        }

        if header.lookup_column_count > 1 {
            return Err(io::Error::new(io::ErrorKind::Other, "lookup_column_count is greater than 1"));
        }

        if header.field_count as usize != W::num_fields() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("UNEXPECTED FIELD COUNT: expect {:?} got {:?}", W::num_fields(), header.field_count),
            ));
        }

        let field_data = FieldStructure::init_from_reader(&mut buf, header.field_count)?;
        if header.field_count as usize != field_data.len() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "field_data size mismatch! field_data size {} should equal to header's field_count {}",
                    field_data.len(),
                    header.field_count,
                ),
            ));
        }
        let data_start = mem::size_of::<WDC1Header>() + mem::size_of::<FieldStructure>() * header.field_count as usize;

        let file_data = if header.has_offset_map_flag() {
            let mut variable_record_data = vec![0u8; header.offset_map_offset as usize - data_start];
            buf.read_exact(&mut variable_record_data)?;
            let number_of_catalogue_entries = (header.max_id - header.min_id + 1).try_into().unwrap();
            let mut offset_map = vec![];
            for _ in 0..number_of_catalogue_entries {
                offset_map.push(OffsetMapEntry {
                    offset: buf.read_u32::<LittleEndian>()?,
                    size:   buf.read_u16::<LittleEndian>()?,
                })
            }
            FileLoaderData::OffsetMaps {
                variable_record_data,
                offset_map,
                number_of_catalogue_entries,
            }
        } else {
            let mut regular_records_data = vec![0u8; header.record_count as usize * header.record_size as usize];
            let mut strings_table = vec![0u8; header.string_table_size.try_into().unwrap()];
            buf.read_exact(&mut regular_records_data)?;
            buf.read_exact(&mut strings_table)?;
            FileLoaderData::Regular {
                regular_records_data,
                strings_table,
            }
        };

        if W::id_index().is_some() && header.id_list_size != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("header mismatch! id_list_size {} should equal to 0 if NonInlinedIDs", header.id_list_size,),
            ));
        }
        if W::id_index().is_none() && header.id_list_size != header.record_count * 4 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "header mismatch! id_list_size {} should equal to 4 * record_count {} if DB2 has InlinedIDs",
                    header.id_list_size, header.record_count
                ),
            ));
        }

        let mut id_list = vec![];
        for _ in 0..(header.id_list_size / 4) {
            id_list.push(buf.read_u32::<LittleEndian>()?);
        }
        let mut copy_table = vec![];
        for _ in 0..header.copy_table_size as usize / mem::size_of::<DB2RecordCopy>() {
            copy_table.push(DB2RecordCopy {
                id_of_new_row:    buf.read_u32::<LittleEndian>()?,
                id_of_copied_row: buf.read_u32::<LittleEndian>()?,
            })
        }
        let field_storage_infos = FieldStorageInfo::init_from_reader(&header, &mut buf)?;
        if !field_storage_infos.is_empty() && field_data.len() != field_storage_infos.len() {
            // For some DB2 like SpellItemEnchantmentCondition.db2, there is no field_storage_info, but there is field_structure
            // This check ensures that if it exists, the field_structure and field_storage_infos tally.
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "field_storage_info size mismatch! field_storage_info size {} should equal to field_data size {}",
                    field_storage_infos.len(),
                    field_data.len(),
                ),
            ));
        }

        let mut calculated_pallet_data_size = 0;
        let mut calculated_common_data_size = 0;
        let mut pallet_data = BTreeMap::new();
        let mut pallet_array_data = BTreeMap::new();
        let mut common_data = BTreeMap::new();
        for (i, fsi) in field_storage_infos.iter().enumerate() {
            if fsi.additional_data_size == 0 {
                continue;
            }
            match fsi.storage_type {
                FieldCompression::BitpackedIndexed { .. } => {
                    let data = pallet_data.entry(i).or_insert(vec![]);
                    let mut to_read = fsi.additional_data_size as usize;
                    while to_read > 0 {
                        let mut d = [0u8; 4];
                        buf.read_exact(&mut d)?;
                        data.push(d);
                        to_read -= d.len();
                        calculated_pallet_data_size += d.len();
                    }
                },
                FieldCompression::BitpackedIndexedArray { array_count, .. } => {
                    let data = pallet_array_data.entry(i).or_insert(vec![]);
                    let mut to_read = fsi.additional_data_size as usize;
                    while to_read > 0 {
                        let mut pallet = vec![];
                        for _ in 0..array_count {
                            let mut d = [0u8; 4];
                            buf.read_exact(&mut d)?;
                            pallet.push(d);
                            to_read -= d.len();
                            calculated_pallet_data_size += d.len();
                        }
                        data.push(pallet);
                    }
                },
                FieldCompression::CommonData { .. } => {
                    let data = common_data.entry(i).or_insert(BTreeMap::new());
                    let mut to_read = fsi.additional_data_size as usize;
                    while to_read > 0 {
                        let record_id = buf.read_u32::<LittleEndian>()?;
                        let mut d = [0u8; 4];
                        buf.read_exact(&mut d)?;
                        to_read -= mem::size_of_val(&record_id) + d.len();
                        calculated_common_data_size += mem::size_of_val(&record_id) + d.len();
                        data.entry(record_id).or_insert(d);
                    }
                },
                _ => continue,
            }
        }

        if calculated_pallet_data_size != header.pallet_data_size as usize {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "pallet data size mismatch! calculated size {} should equal to header's pallet_data_size {}",
                    calculated_pallet_data_size, header.pallet_data_size,
                ),
            ));
        }

        if calculated_common_data_size != header.common_data_size as usize {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "common data size mismatch! calculated size {} should equal to header's common_data_size {}",
                    calculated_common_data_size, header.common_data_size,
                ),
            ));
        }

        let relationship_mappings = RelationshipMapping::init_from_reader(&header, &mut buf)?;

        let mut buf_remain = vec![];
        buf.read_to_end(&mut buf_remain)?;
        if !buf_remain.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "SANITY_CHECK: somehow file isn't fully consumed. please check again! {} bytes left",
                    buf_remain.len(),
                ),
            ));
        }
        Ok(Self {
            locale,
            header,
            field_data,
            id_list,
            copy_table,
            field_storage_infos,
            pallet_data,
            pallet_array_data,
            common_data,
            relationship_mappings,
            file_data,
            data_start,
            wdc1: PhantomData,
        })
    }

    fn get_num_records_to_iterate(&self) -> usize {
        match &self.file_data {
            FileLoaderData::OffsetMaps {
                number_of_catalogue_entries, ..
            } => *number_of_catalogue_entries,
            FileLoaderData::Regular { .. } => self.header.record_count as usize,
        }
    }

    #[allow(clippy::type_complexity)]
    fn get_raw_record_data(&self, record_number: usize) -> io::Result<Option<(&[u8], BTreeMap<usize, BTreeMap<usize, usize>>)>> {
        let mut field_and_array_offsets = BTreeMap::new();
        let res = match &self.file_data {
            FileLoaderData::OffsetMaps {
                variable_record_data,
                offset_map,
                ..
            } => {
                if offset_map[record_number].offset == 0 || offset_map[record_number].size == 0 {
                    None
                } else {
                    let record_offset_start = offset_map[record_number].offset as usize - self.data_start;
                    let raw_data = &variable_record_data[record_offset_start..record_offset_start + offset_map[record_number].size as usize];
                    let mut offset_relative_to_record = 0;
                    for (field, (field_type, arity)) in &W::db2_fields() {
                        let arity_map = field_and_array_offsets.entry(*field).or_insert(BTreeMap::new());
                        for arr in 0..*arity {
                            arity_map.entry(arr).or_insert(offset_relative_to_record);
                            if let Some(s) = field_type.field_size() {
                                offset_relative_to_record += s;
                            } else {
                                // sanity check, None should be String only
                                if !matches!(field_type, DB2FieldType::String) {
                                    return Err(io::Error::new(
                                        io::ErrorKind::Other,
                                        format!(
                                            "SANITY_CHECK: get_raw_record_data failed as field type is invalid. Expected {:?} but got {:?}",
                                            DB2FieldType::String,
                                            field_type,
                                        ),
                                    ));
                                }

                                let cstr_size = raw_data[offset_relative_to_record..]
                                    .iter()
                                    .fold_while(0usize, |acc, b| {
                                        let acc = acc + 1;
                                        if *b == 0 {
                                            FoldWhile::Done(acc)
                                        } else {
                                            FoldWhile::Continue(acc)
                                        }
                                    })
                                    .into_inner();
                                offset_relative_to_record += cstr_size
                            }
                        }
                    }
                    Some((raw_data, field_and_array_offsets))
                }
            },
            FileLoaderData::Regular { regular_records_data, .. } => {
                let record_offset_start = record_number * self.header.record_size as usize;
                let raw_data = &regular_records_data[record_offset_start..record_offset_start + self.header.record_size as usize];
                Some((raw_data, field_and_array_offsets))
            },
        };
        Ok(res)
    }

    /// Produces the entire contents of the DB2 file. returning a BTreeMap of db2 ids to their respective records
    pub fn produce_data(&self) -> io::Result<BTreeMap<u32, W>> {
        let mut res = BTreeMap::new();
        let mut id_value_to_record_number = BTreeMap::new();
        let max_records = self.get_num_records_to_iterate();
        for record_number in 0..max_records {
            let (raw_record, offset_map_field_and_array_offsets) = match self.get_raw_record_data(record_number)? {
                None => continue,
                Some(d) => d,
            };

            let id_value = self.record_get_id(raw_record, &offset_map_field_and_array_offsets, record_number)?;
            let mut fields = BTreeMap::new();
            for (field_idx, (typ, arity)) in &W::db2_fields() {
                let f = match typ {
                    DB2FieldType::I64 => {
                        let mut vs = vec![];
                        for array_idx in 0..*arity {
                            vs.push(self.record_get_var_num(
                                record_number,
                                &offset_map_field_and_array_offsets,
                                raw_record,
                                *field_idx,
                                array_idx,
                                Self::read_i64,
                            )?)
                        }
                        DB2Field::I64(vs)
                    },
                    DB2FieldType::I32 => {
                        let mut vs = vec![];
                        for array_idx in 0..*arity {
                            vs.push(self.record_get_var_num(
                                record_number,
                                &offset_map_field_and_array_offsets,
                                raw_record,
                                *field_idx,
                                array_idx,
                                Self::read_i32,
                            )?);
                        }
                        DB2Field::I32(vs)
                    },
                    DB2FieldType::I16 => {
                        let mut vs = vec![];
                        for array_idx in 0..*arity {
                            vs.push(self.record_get_var_num(
                                record_number,
                                &offset_map_field_and_array_offsets,
                                raw_record,
                                *field_idx,
                                array_idx,
                                Self::read_i16,
                            )?);
                        }
                        DB2Field::I16(vs)
                    },
                    DB2FieldType::I8 => {
                        let mut vs = vec![];
                        for array_idx in 0..*arity {
                            vs.push(self.record_get_var_num(
                                record_number,
                                &offset_map_field_and_array_offsets,
                                raw_record,
                                *field_idx,
                                array_idx,
                                Self::read_i8,
                            )?);
                        }
                        DB2Field::I8(vs)
                    },
                    DB2FieldType::U64 => {
                        let mut vs = vec![];
                        for array_idx in 0..*arity {
                            vs.push(self.record_get_var_num(
                                record_number,
                                &offset_map_field_and_array_offsets,
                                raw_record,
                                *field_idx,
                                array_idx,
                                Self::read_u64,
                            )?);
                        }
                        DB2Field::U64(vs)
                    },
                    DB2FieldType::U32 => {
                        let mut vs = vec![];
                        for array_idx in 0..*arity {
                            vs.push(self.record_get_var_num(
                                record_number,
                                &offset_map_field_and_array_offsets,
                                raw_record,
                                *field_idx,
                                array_idx,
                                Self::read_u32,
                            )?);
                        }
                        DB2Field::U32(vs)
                    },
                    DB2FieldType::U16 => {
                        let mut vs = vec![];
                        for array_idx in 0..*arity {
                            vs.push(self.record_get_var_num(
                                record_number,
                                &offset_map_field_and_array_offsets,
                                raw_record,
                                *field_idx,
                                array_idx,
                                Self::read_u16,
                            )?);
                        }
                        DB2Field::U16(vs)
                    },
                    DB2FieldType::U8 => {
                        let mut vs = vec![];
                        for array_idx in 0..*arity {
                            vs.push(self.record_get_var_num(
                                record_number,
                                &offset_map_field_and_array_offsets,
                                raw_record,
                                *field_idx,
                                array_idx,
                                Self::read_u8,
                            )?);
                        }
                        DB2Field::U8(vs)
                    },
                    DB2FieldType::F32 => {
                        let mut vs = vec![];
                        for array_idx in 0..*arity {
                            vs.push(self.record_get_var_num(
                                record_number,
                                &offset_map_field_and_array_offsets,
                                raw_record,
                                *field_idx,
                                array_idx,
                                Self::read_f32,
                            )?);
                        }
                        DB2Field::F32(vs)
                    },
                    DB2FieldType::String => {
                        let mut vs = vec![];
                        for array_idx in 0..*arity {
                            let mut s = new_localised_string();
                            s[self.locale as usize] =
                                self.record_get_string(raw_record, &offset_map_field_and_array_offsets, record_number, *field_idx, array_idx)?;
                            vs.push(s);
                        }
                        DB2Field::String(vs)
                    },
                };
                fields.entry(*field_idx).or_insert(f);
            }
            res.entry(record_number).or_insert(DB2RawRecord {
                id: id_value,
                fields,
                parent: None,
            });
            id_value_to_record_number.entry(id_value).or_insert(record_number);
        }

        if let Some(typ) = W::non_inline_parent_index_type() {
            for RelationshipData { foreign_id, record_index } in &self.relationship_mappings[0].entries {
                let record_index: usize = (*record_index).try_into().unwrap();
                let mut record = match res.get_mut(&record_index) {
                    None => continue,
                    Some(r) => r,
                };
                let parent_id = match typ {
                    DB2FieldType::I32 => DB2Field::I32(vec![*foreign_id as i32]),
                    DB2FieldType::I16 => DB2Field::I16(vec![*foreign_id as i16]),
                    DB2FieldType::I8 => DB2Field::I8(vec![*foreign_id as i8]),
                    DB2FieldType::U32 => DB2Field::U32(vec![*foreign_id]),
                    DB2FieldType::U16 => DB2Field::U16(vec![*foreign_id as u16]),
                    DB2FieldType::U8 => DB2Field::U8(vec![*foreign_id as u8]),
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("SANITY_CHECK: parent_id type invalid. please check again! got type {typ:?}",),
                        ));
                    },
                };
                record.parent = Some(parent_id)
            }
        }

        let mut new_record_number = max_records;
        for copy in &self.copy_table {
            // Should exist, go ahead and panic if not
            let record_num = id_value_to_record_number.get(&copy.id_of_copied_row).unwrap();
            let mut copied_row = res.get(record_num).unwrap().clone();
            copied_row.id = copy.id_of_new_row;
            res.entry(new_record_number).or_insert(copied_row);
            new_record_number += 1;
        }
        let res = res.into_values().map(|record| (record.id, record.into())).collect::<BTreeMap<_, _>>();
        Ok(res)
    }
}

macro_rules! read_pallet_value {
    ( $raw:expr, $bitpack_bytes_offset:expr, $record_field_bit_offset:expr, $record_field_bit_size:expr ) => {{
        use std::cmp::min;

        use byteorder::LittleEndian;
        let offset = ($record_field_bit_offset / 8 + $bitpack_bytes_offset) as usize;
        let bits_to_read = $record_field_bit_offset as u64 & 7;
        let size_mask = (1 << $record_field_bit_size) - 1;
        let mut buf = [0u8; 8];
        let max_len_to_cpy = min(8, $raw.len() - offset);
        buf[..max_len_to_cpy].clone_from_slice(&$raw[offset..offset + max_len_to_cpy]);
        (LittleEndian::read_u64(&buf) >> bits_to_read) & size_mask
    }};
}

/// Getter methods for getting supported data types
impl<W> FileLoader<W>
where
    W: WDC1,
{
    fn read_i64(raw: &[u8]) -> io::Result<i64> {
        io::Cursor::new(raw).read_i64::<LittleEndian>()
    }

    fn read_i32(raw: &[u8]) -> io::Result<i32> {
        io::Cursor::new(raw).read_i32::<LittleEndian>()
    }

    fn read_i16(raw: &[u8]) -> io::Result<i16> {
        io::Cursor::new(raw).read_i16::<LittleEndian>()
    }

    fn read_i8(raw: &[u8]) -> io::Result<i8> {
        io::Cursor::new(raw).read_i8()
    }

    fn read_u64(raw: &[u8]) -> io::Result<u64> {
        io::Cursor::new(raw).read_u64::<LittleEndian>()
    }

    fn read_u32(raw: &[u8]) -> io::Result<u32> {
        io::Cursor::new(raw).read_u32::<LittleEndian>()
    }

    fn read_u16(raw: &[u8]) -> io::Result<u16> {
        io::Cursor::new(raw).read_u16::<LittleEndian>()
    }

    fn read_u8(raw: &[u8]) -> io::Result<u8> {
        io::Cursor::new(raw).read_u8()
    }

    fn read_f32(raw: &[u8]) -> io::Result<f32> {
        io::Cursor::new(raw).read_f32::<LittleEndian>()
    }

    fn cstr_bytes_to_string(raw: &[u8]) -> io::Result<String> {
        match CStr::from_bytes_until_nul(raw) {
            Err(err) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("ERROR: can't convert to str, bytes was {raw:?}; err = {err}"),
            )),
            Ok(c) => Ok(c.to_string_lossy().to_string()),
        }
    }

    fn get_field_storage_data(&self, field_idx: usize) -> (&FieldStructure, Option<&FieldStorageInfo>) {
        let fsi = if self.field_storage_infos.is_empty() {
            None
        } else {
            Some(&self.field_storage_infos[field_idx])
        };

        (&self.field_data[field_idx], fsi)
    }

    pub fn record_get_id(
        &self,
        raw_record: &[u8],
        offset_map_field_and_array_offsets: &BTreeMap<usize, BTreeMap<usize, usize>>,
        record_number: usize,
    ) -> io::Result<u32> {
        if let Some(id_idx) = W::id_index() {
            return self.record_get_var_num(record_number, offset_map_field_and_array_offsets, raw_record, id_idx, 0, Self::read_u32);
        }
        Ok(match &self.file_data {
            FileLoaderData::Regular { .. } => self.id_list[record_number],
            FileLoaderData::OffsetMaps { .. } => self.header.min_id + record_number as u32,
        })
    }

    pub fn record_get_string(
        &self,
        raw_record: &[u8],
        offset_map_field_and_array_offsets: &BTreeMap<usize, BTreeMap<usize, usize>>,
        record_number: usize,
        field_idx: usize,
        array_idx: usize,
    ) -> io::Result<String> {
        let s = match &self.file_data {
            FileLoaderData::Regular { strings_table, .. } => {
                let string_offset = self
                    .record_get_var_num(
                        record_number,
                        offset_map_field_and_array_offsets,
                        raw_record,
                        field_idx,
                        array_idx,
                        Self::read_u32,
                    )?
                    .try_into()
                    .unwrap();
                Self::cstr_bytes_to_string(&strings_table[string_offset..])?
            },
            FileLoaderData::OffsetMaps { .. } => {
                let offset = *offset_map_field_and_array_offsets
                    .get(&field_idx)
                    .and_then(|arr_map| arr_map.get(&array_idx))
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("record_get_var_num_sparse: record_number {record_number}, cannot find offset: idx {field_idx} array_idx {array_idx}"),
                        )
                    })?;
                Self::cstr_bytes_to_string(&raw_record[offset..])?
            },
        };
        Ok(s)
    }

    fn record_get_var_num_regular<I>(
        &self,
        record_number: usize,
        offset_map_field_and_array_offsets: &BTreeMap<usize, BTreeMap<usize, usize>>,
        raw_record: &[u8],
        field_idx: usize,
        array_idx: usize,
        convert_func: fn(data_to_convert: &[u8]) -> io::Result<I>,
    ) -> io::Result<I> {
        let (field_struct, fsi) = self.get_field_storage_data(field_idx);
        let res = if let Some(fsi) = fsi {
            match fsi.storage_type {
                FieldCompression::BitpackedInlined { offset_bits, size_bits, .. } => {
                    if array_idx != 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "record_get_var_num_regular: record_number {}, array_idx is not zero for bitpacked inlined field. field idx {:?}, array_idx {:?}",
                                record_number, field_idx, array_idx,
                            ),
                        ));
                    }

                    let packed_value = read_pallet_value!(raw_record, self.header.bitpacked_data_offset, offset_bits, size_bits);
                    // if flags > 0 {
                    // TODO: maybe no need? the packed value is purely bit shifted u64
                    //     // Is signed
                    //     let mask = 1 << (fsi.field_size_bits - 1);
                    //     packed_value = (packed_value ^ mask) - mask;
                    // };
                    let mut data = [0; 8];
                    LittleEndian::write_u64(&mut data[..], packed_value);
                    convert_func(&data)?
                },
                FieldCompression::CommonData { default_value } => {
                    if array_idx != 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "record_get_var_num_regular: record_id {}, array_idx is not zero for common data field. field idx {:?}, array_idx {:?}",
                                record_number, field_idx, array_idx,
                            ),
                        ));
                    }
                    let record_id = self.record_get_id(raw_record, offset_map_field_and_array_offsets, record_number)?;

                    let mut def = [0; 4];
                    let data = match self.common_data.get(&field_idx).and_then(|e| e.get(&record_id)) {
                        None => {
                            LittleEndian::write_u32(&mut def, default_value);
                            &def
                        },
                        Some(o) => o,
                    };
                    convert_func(&data[..])?
                },
                FieldCompression::BitpackedIndexed { offset_bits, size_bits } => {
                    if array_idx != 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "record_get_var_num_regular: record_number {}, array_idx is not zero for bitpacked indexed field. field idx {:?}, array_idx {:?}",
                                record_number, field_idx, array_idx,
                            ),
                        ));
                    }
                    let packed_index: usize = read_pallet_value!(raw_record, self.header.bitpacked_data_offset, offset_bits, size_bits)
                        .try_into()
                        .unwrap();
                    match self.pallet_data.get(&field_idx) {
                        None => {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!(
                                    "record_get_var_num_regular: error getting bitpacked indexed data field, record_number {} field_idx {:?}",
                                    record_number, field_idx,
                                ),
                            ));
                        },
                        Some(data) => convert_func(&data[packed_index][..])?,
                    }
                },
                FieldCompression::BitpackedIndexedArray {
                    offset_bits,
                    size_bits,
                    array_count,
                } => {
                    if array_idx >= array_count.try_into().unwrap() {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "record_get_var_num_regular: error getting bitpacked indexed data array field by index, record_number {} field_idx {:?} array_idx {}, array_count {}",
                                record_number, field_idx, array_idx, array_count
                            ),
                        ));
                    }
                    let packed_index: usize = read_pallet_value!(raw_record, self.header.bitpacked_data_offset, offset_bits, size_bits)
                        .try_into()
                        .unwrap();
                    match self.pallet_array_data.get(&field_idx) {
                        None => {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!(
                                    "record_get_var_num_regular: error getting bitpacked indexed array data field, record_number {} field_idx {:?} array_idx {}",
                                    record_number, field_idx, array_idx,
                                ),
                            ));
                        },
                        Some(pallet_array) => convert_func(&pallet_array[packed_index][array_idx][..])?,
                    }
                },
                FieldCompression::None => {
                    // field size in bytes
                    let field_size = ((32 - field_struct.size) / 8) as usize;
                    let offset = fsi.field_offset_bits as usize / 8 + field_size * array_idx;
                    convert_func(&raw_record[offset..offset + field_size])?
                },
            }
        } else {
            // Similar to FieldCompression::None, but we can't guess the arity
            // field size in bytes
            let field_size = ((32 - field_struct.size) / 8) as usize;
            let offset = field_struct.position as usize / 8 + field_size * array_idx;

            convert_func(&raw_record[offset..offset + field_size])?
        };

        Ok(res)
    }

    fn record_get_var_num_sparse<I>(
        &self,
        record_number: usize,
        offset_map_field_and_array_offsets: &BTreeMap<usize, BTreeMap<usize, usize>>,
        raw_record: &[u8],
        field_idx: usize,
        array_idx: usize,
        convert_func: fn(data_to_convert: &[u8]) -> io::Result<I>,
    ) -> io::Result<I> {
        let (field_struct, ..) = self.get_field_storage_data(field_idx);
        let field_size = ((32 - field_struct.size) / 8) as usize;
        let offset = *offset_map_field_and_array_offsets
            .get(&field_idx)
            .and_then(|arr_map| arr_map.get(&array_idx))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("record_get_var_num_sparse: record_number {record_number}, cannot find offset: idx {field_idx} array_idx {array_idx}"),
                )
            })?;
        convert_func(&raw_record[offset..offset + field_size])
    }

    /// record_get_var_num retrieves a numeric value
    fn record_get_var_num<I>(
        &self,
        record_number: usize,
        offset_map_field_and_array_offsets: &BTreeMap<usize, BTreeMap<usize, usize>>,
        raw_record: &[u8],
        field_idx: usize,
        array_idx: usize,
        convert_func: fn(data_to_convert: &[u8]) -> io::Result<I>,
    ) -> io::Result<I> {
        if field_idx >= self.header.field_count as usize {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "record_get_var_num: record_number {}, Field Idx is greater than field count: idx {:?} field_count {:?}",
                    record_number, field_idx, self.header.field_count
                ),
            ));
        }
        match self.file_data {
            FileLoaderData::OffsetMaps { .. } => self.record_get_var_num_sparse(
                record_number,
                offset_map_field_and_array_offsets,
                raw_record,
                field_idx,
                array_idx,
                convert_func,
            ),
            FileLoaderData::Regular { .. } => self.record_get_var_num_regular(
                record_number,
                offset_map_field_and_array_offsets,
                raw_record,
                field_idx,
                array_idx,
                convert_func,
            ),
        }
    }
}
