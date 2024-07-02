use std::future::Future;

use azothacore_common::AzResult;
use azothacore_database::database_env::WorldDatabase;
use bevy::prelude::*;

use super::WorldError;

pub trait WorldTrait {
    fn is_stopped(&self) -> bool;
    /// Initialize config values
    fn load_config_settings(&mut self, reload: bool) -> impl Future<Output = ()> + Send;
    /// Initialize the World
    fn set_initial_world_settings(&mut self) -> impl Future<Output = Result<(), WorldError>> + Send;
    fn stop_now(&mut self, exit_code: i32) -> Result<i32, WorldError>;
}

#[derive(Resource, sqlx::FromRow)]
pub struct WorldDbVersion {
    pub db_version:      String,
    pub cache_id:        u32,
    pub hotfix_cache_id: u32,
}

impl WorldDbVersion {
    pub async fn load(db: &WorldDatabase) -> AzResult<Option<Self>> {
        let res = sqlx::query_as("SELECT db_version, cache_id, hotfix_cache_id FROM version LIMIT 1")
            .fetch_optional(&**db)
            .await?;
        Ok(res)
    }
}
