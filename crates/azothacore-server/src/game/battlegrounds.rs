use std::sync::atomic::{AtomicU32, Ordering};

use bevy::prelude::Resource;

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum BattlegroundQueueInvitationType {
    #[default]
    NoBalance = 0, // no balance: N+M vs N players
    Balanced = 1, // teams balanced: N+1 vs N players
    Even = 2,     // teams even: N vs N players
}

#[derive(Resource)]
pub struct ArenaTeamMgr {
    next_arena_team_id: AtomicU32,
}

impl Default for ArenaTeamMgr {
    fn default() -> Self {
        Self { next_arena_team_id: 1.into() }
    }
}

impl ArenaTeamMgr {
    /// SetNextArenaTeamId
    pub fn set_next_arena_team_id(&self, id: u32) {
        self.next_arena_team_id.store(id, Ordering::SeqCst);
    }
}
