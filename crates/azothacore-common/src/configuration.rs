use std::{env, future::Future, path::Path};

use convert_case::{Case, Casing};
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{debug, error, info, instrument};

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

    fn _get_option<T, F>(&self, key: &str, fallback: F) -> ConfigGetResult<T>
    where
        T: serde::de::DeserializeOwned,
        F: FnOnce() -> Option<T>,
    {
        let env_var_name = format!("AZ_{}", key.replace(|c: char| !c.is_alphanumeric(), "_")).to_case(Case::UpperSnake);
        let env_var_err: ConfigGetError = match env::var(&env_var_name) {
            Ok(env_value) => {
                debug!(target:"server::loading", "> Config: Found config value '{key}' from environment variable '{env_var_name}'.");
                match serde_json::from_str(&env_value) {
                    Ok(v) => return Ok(v),
                    Err(e) => e.into(),
                }
            },
            Err(e) => e.into(),
        };

        let original_err = match self
            .config
            .as_ref()
            .expect("expect configuration to be set already before calling get_option")
            .get(key)
        {
            Err(e) => e,
            Ok(v) => return Ok(v),
        };
        if let Some(v) = fallback() {
            error!(target:"server::loading", env_err=%env_var_err, cfg_err=%original_err, "> Config: Missing property or err {key} in config file {}, add \"{key} = XXX\" to this file or define '{env_var_name}' as an environment variable.", self.filename);
            return Ok(v);
        }

        Err(original_err)
    }

    pub fn get<T, F>(&self, key: &str, fallback: F) -> T
    where
        T: serde::de::DeserializeOwned,
        F: FnOnce() -> T,
    {
        self._get_option(key, || Some(fallback())).unwrap()
    }

    pub fn get_option<T>(&self, key: &str) -> ConfigGetResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self._get_option(key, || None)
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

    pub async fn reload<F, Fut>(&mut self, load_module_config_callback: F) -> AzResult<()>
    where
        Fut: Future<Output = AzResult<Vec<String>>> + Send + 'static,
        F: Fn(bool) -> Fut,
    {
        self.load_app_configs()?;
        self.load_modules_configs(true, false, load_module_config_callback).await
    }

    /// Loads the main app configuration. This doesnt load the module configurations
    #[instrument(skip(self))]
    pub fn load_app_configs(&mut self) -> AzResult<()> {
        self.config = Some(toml_from_filepath(&self.filename)?);
        Ok(())
    }

    #[instrument(skip(self, load_module_config_callback))]
    pub async fn load_modules_configs<F, Fut>(&self, is_reload: bool, is_need_print_info: bool, load_module_config_callback: F) -> AzResult<()>
    where
        Fut: Future<Output = AzResult<Vec<String>>> + Send + 'static,
        F: Fn(bool) -> Fut,
    {
        if is_need_print_info {
            info!("Loading Modules Configuration...");
        }
        let script_configs = match load_module_config_callback(is_reload).await {
            Err(e) => {
                if !is_reload {
                    error!("error loading initial module configuration for script. Stop loading!\nError was {}", e,);
                    return Err(e);
                }
                error!(cause=%e, "error loading modules configuration for script.");
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

pub static CONFIG_MGR: AsyncRwLock<ConfigMgr> = AsyncRwLock::const_new(ConfigMgr::new());
