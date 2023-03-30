use std::{collections::BTreeSet, fs, io, path::Path};

use tokio::sync::RwLock;
use tracing::{error, info, instrument};

mod structs;

pub use structs::*;
use thiserror::Error;

use crate::{server::game::scripting::ScriptMgr, GenericResult};

pub struct ConfigMgr {
    filename:        String,
    dry_run:         bool,
    list_of_modules: BTreeSet<String>,
    config:          Option<Config>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("encountered unexpected error accessing file")]
    UnexpectedOSError(#[from] io::Error),
    #[error("generic error: {msg}")]
    Generic { msg: String },
}

/// Get the config file or config
fn config_or_configdist(file_name: &str) -> Result<String, ConfigError> {
    let file_exist = match fs::try_exists(file_name) {
        Err(err) => return Err(ConfigError::UnexpectedOSError(err)),
        Ok(ok) => ok,
    };
    if file_exist {
        return Ok(file_name.to_string());
    }
    let p = Path::new(file_name);
    match p.extension() {
        None => p.with_extension("dist").as_path(),
        Some(e) => {
            let mut s = e.to_os_string();
            s.push(".dist");
            p.with_extension(s).as_path()
        },
    };
    Ok(p.to_str().expect("don't anticipate function to fail here").to_string())
}

impl ConfigMgr {
    const fn new() -> ConfigMgr {
        ConfigMgr {
            filename:        String::new(),
            dry_run:         false,
            list_of_modules: BTreeSet::new(),
            config:          None,
        }
    }

    /// Retrieves the worldserver configuration. It is expected that the worldserver values are set
    pub fn world(&self) -> &WorldserverConfig {
        self.config.as_ref().unwrap().worldserver.as_ref().unwrap()
    }

    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn set_dry_run(&mut self, mode: bool) {
        self.dry_run = mode
    }

    pub fn get_filename(&self) -> &String {
        &self.filename
    }

    /// configures the root filename, and list of modules that are supported
    #[instrument(skip(self, list_of_modules))]
    pub fn configure<Iter: IntoIterator<Item = String>>(&mut self, init_file_name: &str, list_of_modules: Iter) {
        // Sets the default to the dist file if it doesnt exist.
        self.filename = config_or_configdist(init_file_name).unwrap_or_else(|e| {
            panic!(
                "unable to read init_file_name at {} due to reasons other than file not found: {}",
                &init_file_name, e
            )
        });
        self.list_of_modules = list_of_modules.into_iter().collect::<BTreeSet<_>>();
    }

    /// Loads the main app configuration. This doesnt load the module configurations
    #[instrument(skip(self))]
    pub fn load_app_configs(&mut self) -> GenericResult {
        self.config = Some(Config::toml_from_filepath(self.filename.as_str())?);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn load_modules_configs(&mut self, is_reload: bool, is_need_print_info: bool) -> GenericResult {
        // if self.list_of_modules
        if self.list_of_modules.is_empty() {
            return Ok(());
        }
        if is_need_print_info {
            info!("\nLoading Module Configuration...");
        }
        let script_configs = match ScriptMgr::on_load_module_config(is_reload).await {
            Err(e) => {
                if !is_reload {
                    error!(
                        "error loading initial module configuration for script. Stop loading!\nError was {}",
                        e,
                    );
                    return Err(e);
                }
                error!("error loading module configuration for script.\nError was {}.", e,);
                Vec::new()
            },
            Ok(v) => v,
        };

        if is_need_print_info {
            if script_configs.is_empty() {
                info!("> Not found modules config files");
            } else {
                info!("\nUsing modules configuration: {:?}", script_configs);
            }
        }
        Ok(())
    }
}

pub static S_CONFIG_MGR: RwLock<ConfigMgr> = RwLock::const_new(ConfigMgr::new());
