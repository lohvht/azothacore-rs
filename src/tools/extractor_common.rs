pub mod casc_handles;

use std::{
    env,
    ffi::CStr,
    io::{self, Read},
    path::{Path, PathBuf},
};

use flagset::{flags, FlagSet};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;
use tracing::warn;

use crate::{
    az_error,
    common::Locale,
    tools::extractor_common::casc_handles::{CascHandlerError, CascLocale, CascStorageHandle},
    AzResult,
};

flags! {
    pub enum DB2AndMapExtractFlags: u8 {
        Map = 0x1,
        Dbc = 0x2,
        Camera = 0x4,
        GameTables = 0x8,
    }
}

pub fn cstr_bytes_to_string(raw: &[u8]) -> io::Result<String> {
    match CStr::from_bytes_until_nul(raw) {
        Err(err) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("ERROR: can't convert to str, bytes was {raw:?}; err = {err}"),
        )),
        Ok(c) => Ok(c.to_string_lossy().to_string()),
    }
}

/// Gets the fixed plain name from a path-like string.
/// This string is taken from the last part of the path-like string.
pub fn get_fixed_plain_name(p: &str) -> String {
    let mut found_ext = false;
    let plain_name_after_slash = match p.rsplit_once(&['\\', '/']) {
        None => p.to_owned(),
        Some((_, s2)) => s2.to_owned(),
    };
    let mut plain_name = String::with_capacity(plain_name_after_slash.capacity());
    let mut char_reverse_iter = plain_name_after_slash.chars().rev().peekable();

    while let Some(c) = char_reverse_iter.next() {
        if !found_ext {
            found_ext = c == '.';
            plain_name.push(c.to_ascii_lowercase());
            continue;
        }
        let prev_char = char_reverse_iter.peek();
        let is_first_char = prev_char.is_none();
        plain_name.push(if c == ' ' {
            '_'
        } else if (is_first_char || !prev_char.unwrap().is_ascii_alphabetic()) && c.is_ascii_lowercase() {
            c.to_ascii_uppercase()
        } else if c.is_ascii_uppercase() && !is_first_char && prev_char.unwrap().is_ascii_alphabetic() {
            c.to_ascii_lowercase()
        } else {
            c
        });
    }
    plain_name.chars().rev().collect()
}

flags! {
    pub enum RunStagesFlag: u8 {
        DB2Extraction,
        VmapExtraction,
        MmapGeneration,
    }
}

structstruck::strike! {
    #[strikethrough[serde_inline_default]]
    #[strikethrough[derive(Deserialize, DefaultFromSerde, Serialize, Clone, Debug,  PartialEq)]]
    pub struct ExtractorConfig {
        #[serde_inline_default(env::current_dir().unwrap().to_string_lossy().to_string())]
        pub input_path: String,
        #[serde_inline_default(env::current_dir().unwrap().to_string_lossy().to_string())]
        pub output_path: String,
        #[serde(default)]
        pub run_stage_flags: FlagSet<RunStagesFlag>,
        #[serde_inline_default(FlagSet::full())]
        pub locales: FlagSet<Locale>,
        #[serde_inline_default(Db2AndMapExtract::default())]
        pub db2_and_map_extract: struct {
            #[serde_inline_default(FlagSet::full())]
            pub extract_flags: FlagSet<DB2AndMapExtractFlags>,
            #[serde_inline_default(true)]
            pub allow_float_to_int: bool,
            #[serde_inline_default(true)]
            pub allow_height_limit: bool,
            #[serde_inline_default(-2000.0)]
            pub use_min_height: f32,
        },
        #[serde_inline_default(VmapExtractAndGenerate::default())]
        pub vmap_extract_and_generate: struct {
            #[serde_inline_default(false)]
            pub precise_vector_data: bool,
            #[serde_inline_default(false)]
            pub override_cached: bool,
        },
        #[serde_inline_default(MmapPathGenerator::default())]
        pub mmap_path_generator: struct{
            /// If not specified, run for all
            #[serde(default)]
            pub map_id_tile_x_y: Option<MapIdTileXY>,
            #[serde(default)]
            pub file: Option<String>,
            #[serde_inline_default(70.0)]
            pub max_angle: f32,
            #[serde_inline_default(false)]
            pub skip_liquid : bool,
            #[serde_inline_default(false)]
            pub skip_continents : bool,
            #[serde_inline_default(true)]
            pub skip_junk_maps : bool,
            #[serde_inline_default(false)]
            pub skip_battlegrounds : bool,
            #[serde_inline_default(false)]
            pub debug_output : bool,
            #[serde_inline_default(false)]
            pub big_base_unit : bool,
            #[serde(default)]
            pub off_mesh_file_path: Option<String>,
        }
    }
}

impl ExtractorConfig {
    pub fn from_toml<R: io::Read>(rdr: &mut R) -> AzResult<Self> {
        let mut buf = String::new();
        rdr.read_to_string(&mut buf)?;
        let res = toml::from_str(&buf).map_err(|e| az_error!(e))?;
        Ok(res)
    }
}

impl VmapExtractAndGenerate {
    pub fn version_string() -> &'static str {
        "V4.06 2018_02"
    }
}

impl Db2AndMapExtract {
    pub fn should_extract(&self, f: DB2AndMapExtractFlags) -> bool {
        self.extract_flags.contains(f)
    }
}

impl ExtractorConfig {
    pub fn get_casc_storage_handler(&self, locale: Locale) -> Result<CascStorageHandle, CascHandlerError> {
        CascStorageHandle::build(self.input_storage_data_dir(), locale.to_casc_locales())
    }

    pub fn get_installed_locales_mask(&self) -> AzResult<FlagSet<CascLocale>> {
        let storage = self.get_casc_storage_handler(Locale::None)?;

        Ok(storage.get_installed_locales_mask()?)
    }

    pub fn input_storage_data_dir(&self) -> PathBuf {
        Path::new(self.input_path.as_str()).join("Data")
    }

    pub fn output_dbc_path(&self, locale: Locale) -> PathBuf {
        Path::new(self.output_path.as_str()).join("dbc").join(locale.to_string().as_str())
    }

    pub fn output_camera_path(&self) -> PathBuf {
        Path::new(self.output_path.as_str()).join("cameras")
    }

    pub fn output_gametable_path(&self) -> PathBuf {
        Path::new(self.output_path.as_str()).join("gt")
    }

    pub fn output_map_path(&self) -> PathBuf {
        Path::new(self.output_path.as_str()).join("maps")
    }

    pub fn output_vmap_sz_work_dir_wmo(&self) -> PathBuf {
        Path::new(self.output_path.as_str()).join("Buildings")
    }

    pub fn output_vmap_sz_work_dir_wmo_dir_bin(&self) -> PathBuf {
        self.output_vmap_sz_work_dir_wmo().join("dir_bin")
    }

    pub fn output_vmap_sz_work_dir_wmo_tmp_gameobject_models(&self) -> PathBuf {
        self.output_vmap_sz_work_dir_wmo().join("temp_gameobject_models")
    }

    pub fn output_vmap_output_path(&self) -> PathBuf {
        Path::new(self.output_path.as_str()).join("vmaps")
    }

    pub fn output_mmap_path(&self) -> PathBuf {
        Path::new(self.output_path.as_str()).join("mmaps")
    }

    pub fn output_meshes_debug_path(&self) -> PathBuf {
        Path::new(self.output_path.as_str()).join("meshes")
    }
}

#[serde_inline_default]
#[derive(Deserialize, DefaultFromSerde, Serialize, Clone, Debug, PartialEq)]
pub struct MapIdTileXY {
    #[serde(default)]
    pub map_id:   u32,
    #[serde(default)]
    pub tile_x_y: Option<(u16, u16)>,
}

pub fn chunked_data_offsets(chunk_data: &[u8]) -> AzResult<Vec<([u8; 4], usize, usize)>> {
    let mut chunks_offsets = vec![];
    let mut pos = 0;
    while pos < chunk_data.len() {
        let mut fcc: [u8; 4] = chunk_data[pos..pos + 4]
            .try_into()
            .map_err(|e| az_error!("error getting fcc: {e}"))?;
        fcc.reverse();
        let size = u32::from_le_bytes(
            chunk_data[pos + 4..pos + 8]
                .try_into()
                .map_err(|e| az_error!("error getting size: {e}"))?,
        );
        let start = pos + 8;
        let end = start + size as usize;
        chunks_offsets.push((fcc, start, end));
        pos += 8 + size as usize;
    }
    Ok(chunks_offsets)
}

pub struct ChunkedFile {
    pub filename:   PathBuf,
    chunk_data:     Vec<u8>,
    chunks_offsets: Vec<([u8; 4], usize, usize)>,
}

impl ChunkedFile {
    pub fn build<P>(storage: &CascStorageHandle, filename: P) -> AzResult<Self>
    where
        P: AsRef<Path>,
    {
        let mut file = storage.open_file(&filename, CascLocale::All.into())?;
        let file_size = file.get_file_size().map_err(|e| {
            use tracing::error;
            let f_display = filename.as_ref().display();
            error!("ChunkedFile::build: error reading filesize from file {f_display}: {e}");
            e
        })? as usize;

        let mut chunk_data: Vec<u8> = Vec::with_capacity(file_size);
        let read_file_size = file.read_to_end(&mut chunk_data).map_err(|e| {
            use tracing::error;
            let f_display = filename.as_ref().display();
            error!("ChunkedFile::build: error reading file {f_display}: {e}");
            e
        })?;

        if file_size != read_file_size {
            return Err(az_error!(
                "Unexpected file sizes while reading chunked file. expect {file_size}, got {read_file_size}"
            ));
        }
        let chunks_offsets = chunked_data_offsets(&chunk_data)?;

        Ok(Self {
            filename: filename.as_ref().to_owned(),
            chunk_data,
            chunks_offsets,
        })
    }

    pub fn chunks(&self) -> impl Iterator<Item = (&[u8; 4], &[u8])> {
        self.chunks_offsets
            .iter()
            .map(|(fcc, start, end)| (fcc, &self.chunk_data[*start..*end]))
    }
}

pub fn get_dir_contents<'a, P: AsRef<Path> + 'a>(dirpath: P, filter: &str) -> io::Result<impl Iterator<Item = PathBuf> + 'a> {
    let path_glob = dirpath.as_ref().join(filter);
    let path_glob = path_glob.as_os_str().to_str().ok_or(io::Error::new(
        io::ErrorKind::Other,
        format!("No valid str from path: {} for filter {filter}", dirpath.as_ref().display()),
    ))?;
    let paths = glob::glob(path_glob)
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "path pattern error from path: {} for filter {filter}; err {e}",
                    dirpath.as_ref().display()
                ),
            )
        })?
        .filter_map(|g| match g {
            Err(e) => {
                warn!("get_dir_contents error getting glob: err {e}");
                None
            },
            Ok(p) => Some(p),
        });

    Ok(paths)
}
