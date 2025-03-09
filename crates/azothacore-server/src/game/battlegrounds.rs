use std::sync::atomic::AtomicU32;

use azothacore_database::database_env::CharacterDatabase;

use crate::shared::id_generators::{DBIDGenerator, IDGenerator};

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum BattlegroundQueueInvitationType {
    #[default]
    NoBalance = 0, // no balance: N+M vs N players
    Balanced = 1, // teams balanced: N+1 vs N players
    Even = 2,     // teams even: N vs N players
}

pub struct ArenaTeamIDGenMarker;

/// ArenaTeamMgr::NextArenaTeamId in TC / AC
/// contains ArenaTeamMgr::SetNextArenaTeamId, ArenaTeamMgr::GenerateArenaTeamId
pub type ArenaTeamIDGenerator = IDGenerator<ArenaTeamIDGenMarker, AtomicU32, u32>;

impl DBIDGenerator<CharacterDatabase, u32> for ArenaTeamIDGenerator {
    const DB_SELECT_MAX_ID_QUERY: &str = "SELECT CAST(COALESCE(MAX(arenateamid), 0) AS UNSIGNED INT)+1 FROM arena_team";
}
