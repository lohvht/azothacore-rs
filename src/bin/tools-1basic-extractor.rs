#![feature(result_option_inspect)]

use std::{fs, io};

use azothacore_rs::{
    common::{banner, Locale},
    logging::init_logging,
    tools::{
        basic_extractor::main_db2_and_map_extract,
        extractor_common::{ExtractorConfig, RunStagesFlag},
        mmap_generator::main_path_generator,
        vmap4_assembler::main_vmap4_assemble,
        vmap4_extractor::main_vmap4_extract,
    },
    AzResult,
};
use tracing::{error, info};
use walkdir::WalkDir;

fn main() -> AzResult<()> {
    let _wg = init_logging();
    let mut f = fs::File::open("env/dist/etc/extractor.toml")?;
    let args = ExtractorConfig::from_toml(&mut f)?;

    banner::azotha_banner_show("Azothacore Extractor", || {
        info!("Client directory: {}", args.input_path);
        info!("Data directory:   {}", args.output_path);
        info!("rest of config: {:?}", args);
    });

    old_client_check(&args)?;

    let installed_locales_mask = args.get_installed_locales_mask()?;
    let mut first_installed_locale: Option<Locale> = None;
    let mut build = 0;
    for l in args.locales.into_iter() {
        if let Locale::none = l {
            continue;
        }
        if (installed_locales_mask & l.to_casc_locales()).bits() == 0 {
            continue;
        }
        let storage = match args.get_casc_storage_handler(l) {
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
            break;
        }
    }

    let first_installed_locale = if let Some(l) = first_installed_locale {
        l
    } else {
        info!("No locales detected!");
        return Ok(());
    };
    if args.run_stage_flags.contains(RunStagesFlag::DB2Extraction) {
        // MAP & DB2 EXTRACTOR
        main_db2_and_map_extract(&args, first_installed_locale, build)?;
    }

    if args.run_stage_flags.contains(RunStagesFlag::VmapExtraction) {
        // VMAP EXTRACTOR
        let (model_spawns_data, temp_gameobject_models) = main_vmap4_extract(&args, first_installed_locale)?;

        // VMAP ASSEMBLER
        main_vmap4_assemble(&args, model_spawns_data, temp_gameobject_models)?;
    }

    if args.run_stage_flags.contains(RunStagesFlag::MmapGeneration) {
        // Mmap generator
        main_path_generator(&args, first_installed_locale)?;
    }

    Ok(())
}

/// old_client_check checks if there are any MPQ files in the Data directory
/// If it does, returns check
fn old_client_check(args: &ExtractorConfig) -> io::Result<()> {
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
