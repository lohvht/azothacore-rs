use std::collections::BTreeSet;

use bevy::prelude::Component;

use crate::game::{dungeon_finding::lfg::LfgState, entities::object::object_guid::ObjectGuid};

#[derive(Component)]
pub struct LfgGroupData {
    // General
    /// LfgGroupData::m_State in TC / AC
    ///< State if group in LFG
    state:            LfgState,
    /// LfgGroupData::m_OldState in TC / AC
    ///< Old State
    old_state:        LfgState,
    /// LfgGroupData::m_Players in TC / AC
    ///< Players in group
    players:          BTreeSet<ObjectGuid>,
    // Dungeon
    /// LfgGroupData::m_Dungeon in TC / AC
    ///< Dungeon entry
    pub dungeon_id:   u32,
    // Vote Kick
    /// LfgGroupData::m_KicksLeft in TC / AC
    ///< Number of kicks left
    kicks_left:       u8,
    /// LfgGroupData::m_VoteKickActive in TC / AC
    vote_kick_active: bool,
}

pub trait KicksLeftConfig: Send + Sync + 'static {
    fn kicks_left(&self) -> u8;
}

impl LfgGroupData {
    /// act as the constructor of LfgGroupData from TC / AC + LfgGroupData::SetDungeon in TC / AC
    pub fn new<C: KicksLeftConfig>(cfg: &C) -> Self {
        Self {
            old_state:        LfgState::None,
            state:            LfgState::None,
            kicks_left:       cfg.kicks_left(),
            players:          BTreeSet::new(),
            dungeon_id:       0,
            vote_kick_active: false,
        }
    }

    /// LfgGroupData::SetState in TC / AC
    pub fn set_state(&mut self, state: LfgState) {
        match state {
            // LfgState::None => {
            //     self.dungeon_id = None;
            //     self.kicks_left = cfg.kicks_left();
            // },
            LfgState::FinishedDungeon | LfgState::Dungeon => {
                self.old_state = state.clone();
            },
            _ => {},
        }
        self.state = state;
    }
}
