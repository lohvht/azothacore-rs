use azothacore_database::{database_env::WorldDatabase, query_as};
// use divert::dt_set_custom_alloc;
use tracing::info;

use crate::game::world::{world_trait::WorldTrait, WorldError};

pub struct World {
    exit_code:                  Option<i32>,
    db_version:                 Option<String>,
    /// Int Configs
    config_clientcache_version: u32,
}

impl World {
    pub const fn new() -> World {
        World {
            exit_code:                  None,
            db_version:                 None,
            config_clientcache_version: 0,
        }
    }
}

impl WorldTrait for World {
    fn is_stopped(&self) -> bool {
        self.exit_code.is_some()
    }

    async fn set_initial_world_settings(&mut self) -> Result<(), WorldError> {
        //     dt_set_custom_alloc();

        Ok(())
    }

    async fn load_db_version(&mut self) -> Result<(), WorldError> {
        let row = query_as::<_, (String, i32)>("SELECT db_version, cache_id FROM version LIMIT 1")
            .fetch_optional(&WorldDatabase::get())
            .await?;

        let (db_version, conf_cache_version) = match row {
            None => return Ok(()),
            Some(e) => e,
        };

        self.db_version = Some(db_version);
        self.config_clientcache_version = conf_cache_version as u32;

        Ok(())
    }

    fn stop_now(&mut self, exit_code: i32) -> Result<i32, WorldError> {
        if self.is_stopped() {
            return Ok(self.exit_code.unwrap());
        }
        info!("Turning world flag to stopped");
        // if let Some(ct) = &self.cancel_token {
        //     ct.cancel();
        // }
        // self.cancel_token = None;
        self.exit_code = Some(exit_code);
        Ok(exit_code)
    }

    fn get_db_version(&self) -> &String {
        self.db_version.as_ref().expect("DB version should have already been loaded")
    }
}
