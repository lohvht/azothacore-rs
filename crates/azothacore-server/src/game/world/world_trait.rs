use azothacore_common::{
    bevy_app::TokioRuntime,
    configuration::{Config, ConfigMgr},
    AccountTypes,
    AzResult,
};
use azothacore_database::database_env::{LoginDatabase, WorldDatabase};
use bevy::prelude::{Commands, In, Res, ResMut, Resource, SystemSet};

use crate::game::{scripting::script_mgr::ScriptMgr, world::CurrentRealm};

/// World::m_allowedSecurityLevel in TC / World::_allowedSecurityLevel in AC
#[derive(Resource)]
pub struct AllowedSecurityLevel(pub AccountTypes);

pub trait WorldTrait<C: Config>
where
    Self: Resource,
{
    // fn is_stopped(&self) -> bool;
    /// Initialize config values - LoadConfigSettings in TC / AC
    fn load_config_settings(reload: In<bool>, commands: Commands, cfg: ResMut<ConfigMgr<C>>, script_mgr: ScriptMgr);
    fn load_db_allowed_security_level(
        this: Res<Self>,
        commands: Commands,
        rt: Res<TokioRuntime>,
        login_db: Res<LoginDatabase>,
        current_realm: Res<CurrentRealm>,
    );
    fn set_player_security_limit(sec: In<AccountTypes>, this: ResMut<AllowedSecurityLevel>);
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorldSets {
    /// Initialize the World
    /// SystemSet analagous to the world function of the same name in TC / AC
    SetInitialWorldSettings,
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
