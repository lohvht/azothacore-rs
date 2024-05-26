use std::{path::Path, sync::OnceLock};

use futures::future::BoxFuture;
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{error, info, instrument};

#[allow(non_snake_case)]
mod structs;

pub use structs::*;

use crate::AzResult;

type BoxedLoadModuleConfigCallback = Box<dyn Fn(bool) -> BoxFuture<'static, AzResult<Vec<String>>> + Send + Sync>;

pub struct ConfigMgr {
    filename:                    String,
    dry_run:                     bool,
    load_module_config_callback: OnceLock<BoxedLoadModuleConfigCallback>,
}

impl ConfigMgr {
    const fn new() -> ConfigMgr {
        ConfigMgr {
            filename:                    String::new(),
            dry_run:                     false,
            load_module_config_callback: OnceLock::new(),
        }
    }

    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn get_filename(&self) -> &Path {
        Path::new(&self.filename)
    }

    pub fn configure<P>(&mut self, init_file_name: P, dry_run: bool, load_module_config_callback: BoxedLoadModuleConfigCallback)
    where
        P: AsRef<Path>,
    {
        self.filename = init_file_name.as_ref().to_str().unwrap().to_string();
        self.dry_run = dry_run;
        self.load_module_config_callback.get_or_init(|| load_module_config_callback);
    }

    pub async fn reload(&mut self) -> AzResult<()> {
        self.load_app_configs()?;
        self.load_modules_configs(true, false).await
    }

    /// Loads the main app configuration. This doesnt load the module configurations
    #[instrument(skip(self))]
    pub fn load_app_configs(&mut self) -> AzResult<()> {
        todo!()
        // self.config = Some(toml_from_filepath(&self.filename)?);
        // Ok(())
    }

    #[instrument(skip(self))]
    pub async fn load_modules_configs(&self, is_reload: bool, is_need_print_info: bool) -> AzResult<()> {
        if is_need_print_info {
            info!("Loading Modules Configuration...");
        }
        let script_configs = match (self.load_module_config_callback.get().unwrap())(is_reload).await {
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
