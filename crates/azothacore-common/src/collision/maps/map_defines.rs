use std::{
    io,
    path::{Path, PathBuf},
};

use flagset::FlagSet;

use crate::{
    az_error,
    recastnavigation_handles::{DT_NAVMESH_VERSION, RC_WALKABLE_AREA},
    sanity_check_read_all_bytes_from_reader,
    utils::{bincode_deserialise, bincode_serialise, buffered_file_create, buffered_file_open},
    AzResult,
};

const MMAP_MAGIC: &[u8; 4] = b"MMAP"; // 'MMAP'
const MMAP_VERSION: u32 = 10;
const DT_VERSION_IN_USE: u32 = DT_NAVMESH_VERSION as u32;

flagset::flags! {
    /// MmapNavTerrainFlag is a bitflag taken from our user defined AreaID that we're using
    /// for each Recast
    ///
    /// In this case the bit flag is calculated by the following formula:
    ///
    /// 1 << (RC_WALKABLE_AREA - area_id)
    ///
    /// The flags are defined on purpose as below to show the above relationship
    pub enum MmapNavTerrainFlag: u16
    {
        // NAV_EMPTY        = 0x00,
        Ground      = 1 << (RC_WALKABLE_AREA - 63),
        GroundSteep = 1 << (RC_WALKABLE_AREA - 62),
        Water       = 1 << (RC_WALKABLE_AREA - 61),
        MagmaSlime  = 1 << (RC_WALKABLE_AREA - 60),
        // areas 1-59 will be used for destructible areas (currently skipped in vmaps, WMO with flag 1)
        // ground is the highest value to make recast choose ground over water when merging surfaces very
        // close to each other (shallow water would be walkable)
    }
}

impl MmapNavTerrainFlag {
    pub fn flags(self) -> FlagSet<Self> {
        self.into()
    }

    /// Area ID, hacky way of getting it but its between this or defining another enum
    /// just for area ID like TC, which would be annoying to maintain.
    pub fn area_id(&self) -> u8 {
        // This will always be a multiple of 2
        let mut bits = self.flags().bits();
        let mut res = 0;
        while bits > 1 {
            res += 1;
            bits >>= 1;
        }
        RC_WALKABLE_AREA - res
    }

    pub fn from_area_id(area_id: u8) -> FlagSet<Self> {
        if area_id == 0 {
            None.into()
        } else {
            FlagSet::new_truncated(1 << u16::from(RC_WALKABLE_AREA - area_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_sanity_checks_mmap_nav_terrain_flag_area_id_generation() {
        assert_eq!(MmapNavTerrainFlag::Ground.area_id(), RC_WALKABLE_AREA);
        assert_eq!(MmapNavTerrainFlag::GroundSteep.area_id(), RC_WALKABLE_AREA - 1);
        assert_eq!(MmapNavTerrainFlag::Water.area_id(), RC_WALKABLE_AREA - 2);
        assert_eq!(MmapNavTerrainFlag::MagmaSlime.area_id(), RC_WALKABLE_AREA - 3);
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MmapTileFileHeader {
    mmap_magic:   [u8; 4],
    dt_version:   u32,
    mmap_version: u32,
}

impl MmapTileFileHeader {
    pub fn verify(&self) -> AzResult<()> {
        if self.dt_version != DT_VERSION_IN_USE {
            Err(az_error!(
                "Detour version mismatch: got {}, current supports {DT_VERSION_IN_USE}",
                self.dt_version
            ))
        } else if self.mmap_version != MMAP_VERSION {
            Err(az_error!(
                "MMAP version mismatch: got {}, current supports {MMAP_VERSION}",
                self.mmap_version
            ))
        } else if self.mmap_magic != *MMAP_MAGIC {
            Err(az_error!(
                "MMAP magic mismatch: got {}, current supports {}",
                String::from_utf8_lossy(&self.mmap_magic[..]),
                String::from_utf8_lossy(MMAP_MAGIC),
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MmapTileFile {
    pub header: MmapTileFileHeader,
    pub data:   MmapTileData,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MmapTileData {
    pub uses_liquids: bool,
    pub bytes:        Vec<u8>,
}

impl MmapTileFile {
    pub fn new(uses_liquids: bool, data: Vec<u8>) -> MmapTileFile {
        Self {
            header: MmapTileFileHeader {
                dt_version:   DT_VERSION_IN_USE,
                mmap_magic:   *MMAP_MAGIC,
                mmap_version: MMAP_VERSION,
            },
            data:   MmapTileData { uses_liquids, bytes: data },
        }
    }

    // static char const* const MAP_FILE_NAME_FORMAT = "%smmaps/%04i.mmap";
    /// equivalent to the TILE_FILE_NAME_FORMAT in TC/AC
    pub fn mmap_tile_filepath<P: AsRef<Path>>(mmap_dir_path: P, map_id: u32, tile_y: u16, tile_x: u16) -> PathBuf {
        mmap_dir_path.as_ref().join(format!("{map_id:04}{tile_y:02}{tile_x:02}.mmtile"))
    }

    #[tracing::instrument(skip(self, mmap_dir_path))]
    pub fn write_to_mmtile<P: AsRef<Path>>(&self, mmap_dir_path: P, map_id: u32, tile_y: u16, tile_x: u16) -> AzResult<()> {
        let p = Self::mmap_tile_filepath(mmap_dir_path, map_id, tile_y, tile_x);
        // file output
        tracing::info!("Writing to mmtile file {}...", p.display());
        let mut file = match buffered_file_create(&p) {
            Err(e) => {
                return Err(az_error!(
                    "[Map {map_id:04}] Failed to open {} for writing! err was {e}",
                    p.display()
                ))
            },
            Ok(f) => f,
        };
        self.write(&mut file)
    }

    fn write<W: io::Write>(&self, mut w: &mut W) -> AzResult<()> {
        bincode_serialise(&mut w, &self.header)?;
        bincode_serialise(&mut w, &self.data)?;

        Ok(())
    }

    fn read_header<R: io::Read>(mut r: &mut R) -> AzResult<MmapTileFileHeader> {
        let header = bincode_deserialise(&mut r)?;

        Ok(header)
    }

    pub fn read_header_from_mmtile<P: AsRef<Path>>(
        mmap_dir_path: P,
        map_id: u32,
        tile_y: u16,
        tile_x: u16,
    ) -> AzResult<MmapTileFileHeader> {
        let p = Self::mmap_tile_filepath(mmap_dir_path, map_id, tile_y, tile_x);
        let mut file = match buffered_file_open(&p) {
            Err(e) => {
                return Err(az_error!(
                    "[Map {map_id:04}] Failed to open {} for writing! err was {e}",
                    p.display()
                ))
            },
            Ok(f) => f,
        };
        Self::read_header(&mut file)
    }

    pub fn read_from_mmtile<P: AsRef<Path>>(mmap_dir_path: P, map_id: u32, tile_y: u16, tile_x: u16) -> AzResult<Self> {
        let p = Self::mmap_tile_filepath(mmap_dir_path, map_id, tile_y, tile_x);
        let mut file = match buffered_file_open(&p) {
            Err(e) => {
                return Err(az_error!(
                    "[Map {map_id:04}] Failed to open {} for writing! err was {e}",
                    p.display()
                ))
            },
            Ok(f) => f,
        };
        Self::read(&mut file)
    }

    fn read<R: io::Read>(mut r: &mut R) -> AzResult<Self> {
        let header = Self::read_header(&mut r)?;
        header.verify()?;
        let data = bincode_deserialise(&mut r)?;
        sanity_check_read_all_bytes_from_reader!(r)?;
        Ok(Self { header, data })
    }
}
