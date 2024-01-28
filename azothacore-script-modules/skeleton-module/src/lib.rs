use std::{path::PathBuf, sync::Arc};

use azothacore_common::{configuration::toml_from_filepath, AzResult, CONF_MODULES_DIR};
use azothacore_server::game::scripting::script_mgr::{ScriptObject, WorldScript, SCRIPT_MGR};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;
use tracing::{info, instrument};

#[derive(Deserialize, Serialize, Clone, Debug)]
struct MyConfig {
    enabled: bool,
}

static MY_CONFIG: AsyncMutex<MyConfig> = AsyncMutex::const_new(MyConfig { enabled: false });

#[derive(Debug)]
struct MyWorld;

impl ScriptObject for MyWorld {}

impl WorldScript for MyWorld {
    #[instrument(skip(self))]
    fn on_load_module_config(&self, _reload: bool) -> AzResult<Vec<String>> {
        info!("start");

        let p = PathBuf::from(CONF_MODULES_DIR).join("my_conf.toml");
        let mut conf = MY_CONFIG.blocking_lock();
        *conf = toml_from_filepath(&p)?;

        info!(">>> config loaded, test_config was: {:?}", conf);

        Ok(vec![format!("{}", p.display())])
    }
}

pub fn init() -> AzResult<()> {
    let script = MyWorld {};

    let script = Arc::new(script);
    SCRIPT_MGR.blocking_write().register_world_script(script);
    Ok(())
}
