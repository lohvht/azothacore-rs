pub mod casc_handles;

use std::{
    env,
    io,
    path::{Path, PathBuf},
};

use flagset::{flags, FlagSet};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

use crate::{common::Locale, GenericResult};

flags! {
    pub enum DB2AndMapExtractFlags: u8 {
        Map = 0x1,
        Dbc = 0x2,
        Camera = 0x4,
        GameTables = 0x8,
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
            precise_vector_data: bool,
        },
    }
}

impl ExtractorConfig {
    pub fn from_toml<R: io::Read>(rdr: &mut R) -> GenericResult<Self> {
        let mut buf = String::new();
        rdr.read_to_string(&mut buf)?;
        Ok(toml::from_str(&buf)?)
    }
}

impl Db2AndMapExtract {
    pub fn should_extract(&self, f: DB2AndMapExtractFlags) -> bool {
        !(self.extract_flags & f).is_empty()
    }
}

impl ExtractorConfig {
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
}
