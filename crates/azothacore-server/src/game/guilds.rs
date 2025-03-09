use std::sync::atomic::AtomicU64;

use azothacore_database::database_env::CharacterDatabase;

use crate::shared::id_generators::{DBIDGenerator, IDGenerator};

pub struct GuildIDGenMarker;

/// GuildMgr::NextGuildId in TC / AC
/// contains GuildMgr::SetNextGuildId, GuildMgr::GenerateGuildId
pub type GuildIDGenerator = IDGenerator<GuildIDGenMarker, AtomicU64, u64>;

impl DBIDGenerator<CharacterDatabase, u64> for GuildIDGenerator {
    const DB_SELECT_MAX_ID_QUERY: &str = "SELECT CAST(COALESCE(MAX(guildId), 0) AS UNSIGNED INT)+1 FROM guild";
}
