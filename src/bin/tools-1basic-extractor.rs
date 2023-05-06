#![feature(result_option_inspect)]

use std::{
    env,
    fs,
    io::{self},
    path::{Path, PathBuf},
    str::FromStr,
};

use azothacore_rs::{
    common::{banner, Locale, LocaleParseError},
    logging::init_logging,
    server::shared::data_stores::db2_structure::{CinematicCamera, LiquidMaterial, LiquidObject, LiquidType, Map},
    tools::{
        extractor_common::casc_handles::{CascFileHandle, CascHandlerError, CascLocale, CascStorageHandle},
        wow7_3_5_26972::basic_extractor::DB_FILES_CLIENT_LIST,
    },
    GenericResult,
};
use clap::Parser;
use flagset::{flags, FlagSet};
use tracing::{error, info};
use walkdir::WalkDir;
use wow_db2::wdc1;

flags! {
    enum ExtractFlags: u8 {
        Map = 0x1,
        Dbc = 0x2,
        Camera = 0x4,
        GameTables = 0x8,
        All = (ExtractFlags::Map | ExtractFlags::Dbc | ExtractFlags::Camera | ExtractFlags::GameTables).bits(),
    }
}

fn current_dir_as_string() -> String {
    env::current_dir().unwrap().to_string_lossy().to_string()
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t=current_dir_as_string())]
    input_path:         String,
    #[arg(short, long, default_value_t=current_dir_as_string())]
    output_path:        String,
    #[arg(short, long, default_value_t=FlagSet::from(ExtractFlags::All).bits())]
    extract:            u8,
    #[arg(short = 'f', long, default_value_t = true)]
    allow_float_to_int: bool,
    #[arg(long, default_value_t = true)]
    allow_height_limit: bool,
    #[arg(long, default_value_t = -2000.0)]
    use_min_height:     f64,
    #[arg(short, long, default_values_t=vec!["enUS".to_string()])]
    locales:            Vec<String>,
}

impl Args {
    fn input_storage_data_dir(&self) -> PathBuf {
        Path::new(self.input_path.as_str()).join("Data")
    }

    fn output_dbc_path(&self, locale: Locale) -> PathBuf {
        Path::new(self.output_path.as_str()).join("dbc").join(locale.to_string().as_str())
    }

    fn should_extract(&self, f: ExtractFlags) -> bool {
        (self.extract & FlagSet::from(f).bits()) > 0
    }

    fn output_camera_path(&self) -> PathBuf {
        Path::new(self.output_path.as_str()).join("cameras")
    }

    fn output_gametable_path(&self) -> PathBuf {
        Path::new(self.output_path.as_str()).join("gt")
    }

    fn output_map_path(&self) -> PathBuf {
        Path::new(self.output_path.as_str()).join("maps")
    }
}

fn main() -> GenericResult<()> {
    init_logging();
    banner::azotha_banner_show("Map & DBC Extractor (i.e. Basic Extractor)", || {});

    let args = Args::parse();

    old_client_check(&args)?;

    let installed_locales_mask = get_installed_locales_mask(&args)?;
    let mut first_installed_locale: Option<Locale> = None;
    let mut build = 0;

    let arg_locales = args
        .locales
        .iter()
        .map(|v| Locale::from_str(v))
        .collect::<Result<Vec<Locale>, LocaleParseError>>()?;

    for l in arg_locales {
        if let Locale::none = l {
            continue;
        }
        if (installed_locales_mask & l.to_casc_locales()).bits() == 0 {
            continue;
        }
        let storage = match get_casc_storage_handler(&args, l) {
            Err(e) => {
                error!(
                    "error opening casc storage '{}' locale {}, err was {}",
                    args.input_storage_data_dir().display(),
                    l,
                    e,
                );
                continue;
            },
            Ok(r) => r,
        };
        let product_info = match storage.get_product_info() {
            Err(_) => continue,
            Ok(r) => r,
        };
        if first_installed_locale.is_none() {
            build = product_info.build_number;
            first_installed_locale = Some(l);
        }

        if !args.should_extract(ExtractFlags::Dbc) {
            info!("Detected client build: {}", build);
            break;
        }
        // Extract DBC files
        info!("Detected client build: {} for locale {}", build, l);
        extract_db_files_client(&storage, &args, l)?;
    }

    let first_installed_locale = if let Some(l) = first_installed_locale {
        l
    } else {
        info!("No locales detected!");
        return Ok(());
    };

    if args.should_extract(ExtractFlags::Camera) {
        extract_camera_files(&args, first_installed_locale)?;
    }
    if args.should_extract(ExtractFlags::GameTables) {
        extract_game_tables(&args, first_installed_locale)?;
    }
    if args.should_extract(ExtractFlags::Map) {
        extract_maps(&args, first_installed_locale, build)?;
    }

    Ok(())
}

fn get_casc_storage_handler(args: &Args, locale: Locale) -> Result<CascStorageHandle, CascHandlerError> {
    CascStorageHandle::build(args.input_storage_data_dir(), locale.to_casc_locales())
}

fn get_casc_filename_part<P: AsRef<Path>>(casc_path: P) -> PathBuf {
    if let Some(last_sep) = casc_path.as_ref().file_name() {
        Path::new(last_sep).to_path_buf()
    } else {
        casc_path.as_ref().to_path_buf()
    }
}

fn extract_db_files_client(storage: &CascStorageHandle, args: &Args, locale: Locale) -> GenericResult<()> {
    info!("Extracting dbc/db2 files for {}...", locale);
    let locale_path = args.output_dbc_path(locale);

    fs::create_dir_all(&locale_path)?;

    info!("locale {} output path {}", locale, locale_path.display());
    let mut count = 0;

    for file_name in DB_FILES_CLIENT_LIST {
        let mut dbc_file = match storage.open_file(file_name, FlagSet::from(CascLocale::None)) {
            Err(e) => {
                error!("Unable to open file {} in the archive for locale {}: {}", file_name, locale, e);
                continue;
            },
            Ok(r) => r,
        };
        let file_path = locale_path.join(get_casc_filename_part(file_name));
        if file_path.exists() {
            continue;
        }
        if extract_file(&mut dbc_file, file_path).is_err() {
            continue;
        }
        count += 1;
    }
    info!("Extracted {} files!", count);
    Ok(())
}

fn extract_file(file_in_archive: &mut CascFileHandle, out_path: PathBuf) -> GenericResult<()> {
    let file_size = file_in_archive.get_file_size()?;

    let mut output = fs::File::create(&out_path).inspect_err(|e| {
        error!("can't create the output file '{}', err was: {}", out_path.display(), e);
    })?;

    let res = io::copy(file_in_archive, &mut output)?;

    // Sanity check here! just verifying that file_size detected is the same as the result
    if file_size != res {
        let e = Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Extracted file sizes don't match somehow for {}. expected {}, got {}",
                out_path.display(),
                file_size,
                res
            )
            .as_str(),
        ));
        error!("extract_file has failed somehow: {}", e);
        Err(e)
    } else {
        Ok(())
    }
}

fn extract_camera_files(args: &Args, locale: Locale) -> GenericResult<()> {
    info!("Extracting camera files...");

    let storage = get_casc_storage_handler(&args, locale)?;
    let camera_file_names = read_cinematic_camera_dbc(&storage, locale)?;

    let output_path = args.output_camera_path();

    fs::create_dir_all(&output_path)?;

    info!("output camera path is {}", output_path.display());

    // extract M2s
    let mut count = 0;
    for camera_file_name in camera_file_names {
        let mut dbc_file = storage.open_file(&camera_file_name, CascLocale::None.into())?;
        let file_path = output_path.join(get_casc_filename_part(&camera_file_name));
        if file_path.exists() {
            continue;
        }
        if extract_file(&mut dbc_file, file_path).is_err() {
            continue;
        }
        count += 1;
    }
    info!("Extracted {count} camera files");

    Ok(())
}

fn read_cinematic_camera_dbc(storage: &CascStorageHandle, locale: Locale) -> GenericResult<Vec<String>> {
    info!("Read CinematicCamera.db2 file...");
    let source = storage.open_file("DBFilesClient/CinematicCamera.db2", CascLocale::None.into())?;
    let fl = wdc1::FileLoader::<CinematicCamera>::from_reader(source, locale as u32)?;
    let data = fl.produce_data()?;

    let res = data
        .values()
        .map(|d| {
            let fid = d.file_data_id;
            format!("FILE{fid:08X}.xxx")
        })
        .collect::<Vec<_>>();

    info!("Done! ({} CinematicCameras loaded)", res.len());
    Ok(res)
}

fn extract_game_tables(args: &Args, locale: Locale) -> GenericResult<()> {
    info!("Extracting game tables...");
    let storage = get_casc_storage_handler(args, locale)?;
    let output_path = args.output_gametable_path();

    fs::create_dir_all(&output_path)?;

    info!("output game table path is {}", output_path.display());

    let game_tables = [
        "GameTables/ArmorMitigationByLvl.txt",
        "GameTables/ArtifactKnowledgeMultiplier.txt",
        "GameTables/ArtifactLevelXP.txt",
        "GameTables/BarberShopCostBase.txt",
        "GameTables/BaseMp.txt",
        "GameTables/BattlePetTypeDamageMod.txt",
        "GameTables/BattlePetXP.txt",
        "GameTables/ChallengeModeDamage.txt",
        "GameTables/ChallengeModeHealth.txt",
        "GameTables/CombatRatings.txt",
        "GameTables/CombatRatingsMultByILvl.txt",
        "GameTables/HonorLevel.txt",
        "GameTables/HpPerSta.txt",
        "GameTables/ItemSocketCostPerLevel.txt",
        "GameTables/NpcDamageByClass.txt",
        "GameTables/NpcDamageByClassExp1.txt",
        "GameTables/NpcDamageByClassExp2.txt",
        "GameTables/NpcDamageByClassExp3.txt",
        "GameTables/NpcDamageByClassExp4.txt",
        "GameTables/NpcDamageByClassExp5.txt",
        "GameTables/NpcDamageByClassExp6.txt",
        "GameTables/NPCManaCostScaler.txt",
        "GameTables/NpcTotalHp.txt",
        "GameTables/NpcTotalHpExp1.txt",
        "GameTables/NpcTotalHpExp2.txt",
        "GameTables/NpcTotalHpExp3.txt",
        "GameTables/NpcTotalHpExp4.txt",
        "GameTables/NpcTotalHpExp5.txt",
        "GameTables/NpcTotalHpExp6.txt",
        "GameTables/SandboxScaling.txt",
        "GameTables/SpellScaling.txt",
        "GameTables/xp.txt",
    ];

    let mut count = 0;
    for file_name in game_tables {
        let mut dbc_file = storage.open_file(&file_name, CascLocale::None.into())?;
        let file_path = output_path.join(get_casc_filename_part(&file_name));
        if file_path.exists() {
            continue;
        }
        if extract_file(&mut dbc_file, file_path).is_err() {
            continue;
        }
        count += 1;
    }
    info!("Extracted {count} game table files");
    Ok(())
}

fn extract_maps(args: &Args, locale: Locale, build_no: u32) -> GenericResult<()> {
    todo!();
}

fn get_installed_locales_mask(args: &Args) -> GenericResult<FlagSet<CascLocale>> {
    let storage = get_casc_storage_handler(&args, Locale::none)?;

    Ok(storage.get_installed_locales_mask()?)
}

/// old_client_check checks if there are any MPQ files in the Data directory
/// If it does, returns check
fn old_client_check(args: &Args) -> io::Result<()> {
    let storage_dir = args.input_storage_data_dir();
    let has_mpq = WalkDir::new(storage_dir).into_iter().any(|direntry| {
        match direntry {
            Err(err) => {
                error!("Error checking client version due to directory walk error: {}", err.to_string());
                // skip over anyway
                false
            },
            Ok(de) => {
                let r = de.path().extension();
                if let Some(ex) = r {
                    let res = "MPQ" == ex;
                    if res {
                        error!(
                            r#"
                        MPQ files found in Data directory!
                        This tool works only with World of Warcraft: Legion

                        To extract maps for Wrath of the Lich King, rebuild tools using 3.3.5 branch!
                        "#
                        )
                    }
                    res
                } else {
                    // If directory has no extension we shouldnt really care
                    false
                }
            },
        }
    });
    if has_mpq {
        Err(io::Error::new(io::ErrorKind::Other, "HAS_MPQ"))
    } else {
        Ok(())
    }
}
