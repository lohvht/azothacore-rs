use std::{path::Path, sync::RwLock};

use tracing::{error, info, instrument};

#[allow(non_snake_case)]
mod structs;

pub use structs::*;

use crate::AzResult;

pub struct ConfigMgr {
    filename: String,
    dry_run:  bool,
    config:   Option<ConfigTable>,
}

impl ConfigMgr {
    const fn new() -> ConfigMgr {
        ConfigMgr {
            filename: String::new(),
            dry_run:  false,
            config:   None,
        }
    }

    pub fn get_option<'de, T>(&self, key: &str) -> ConfigGetResult<T>
    where
        T: serde::Deserialize<'de>,
    {
        self.config
            .as_ref()
            .expect("expect configuration to be set already before calling get_option")
            .get(key)
    }

    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn get_filename(&self) -> &Path {
        Path::new(&self.filename)
    }

    pub fn configure<P: AsRef<Path>>(&mut self, init_file_name: P, dry_run: bool) {
        self.filename = init_file_name.as_ref().to_str().unwrap().to_string();
        self.dry_run = dry_run;
    }

    pub fn reload<F>(&mut self, load_module_config_callback: F) -> AzResult<()>
    where
        F: Fn(bool) -> AzResult<Vec<String>>,
    {
        self.load_app_configs()?;
        self.load_modules_configs(true, false, load_module_config_callback)
    }

    /// Loads the main app configuration. This doesnt load the module configurations
    #[instrument(skip(self))]
    pub fn load_app_configs(&mut self) -> AzResult<()> {
        self.config = Some(toml_from_filepath(&self.filename)?);
        Ok(())
    }

    #[instrument(skip(self, load_module_config_callback))]
    pub fn load_modules_configs<F>(&self, is_reload: bool, is_need_print_info: bool, load_module_config_callback: F) -> AzResult<()>
    where
        F: Fn(bool) -> AzResult<Vec<String>>,
    {
        if is_need_print_info {
            info!("\nLoading Modules Configuration...");
        }
        let script_configs = match load_module_config_callback(is_reload) {
            Err(e) => {
                if !is_reload {
                    error!(
                        "error loading initial module configuration for script. Stop loading!\nError was {}",
                        e,
                    );
                    return Err(e);
                }
                error!("error loading modules configuration for script.\nError was {}.", e,);
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

pub static CONFIG_MGR: RwLock<ConfigMgr> = RwLock::new(ConfigMgr::new());