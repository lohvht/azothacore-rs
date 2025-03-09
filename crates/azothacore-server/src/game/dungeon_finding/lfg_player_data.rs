use std::collections::BTreeSet;

use bevy::prelude::Component;

use crate::{
    game::{
        dungeon_finding::lfg::LfgState,
        entities::object::object_guid::{HighGuidParty, ObjectGuid},
        server::world_packets::lfg_packets_common::LFGRideTicket,
    },
    shared::shared_defines::FactionID,
};

#[derive(Component)]
pub struct LfgPlayerData {
    // General
    /// LfgPlayerData::m_Ticket in TC
    ///
    ///< Join ticket
    ticket:            LFGRideTicket,
    /// LfgPlayerData::m_State in TC / AC
    ///< State if group in LFG
    state:             LfgState,
    /// LfgPlayerData::m_OldState in TC / AC
    ///< Old State
    old_state:         LfgState,
    // Player
    /// LfgPlayerData::m_Team in TC / AC
    ///< Player team - determines the queue to join
    team:              FactionID,
    /// LfgPlayerData::m_Group in TC / AC
    ///< Original group of player when joined LFG
    group:             ObjectGuid<HighGuidParty>,
    // Queue
    /// LfgPlayerData::m_Roles in TC / AC
    ///< Roles the player selected when joined LFG
    roles:             u8,
    /// LfgPlayerData::m_SelectedDungeons in TC / AC
    ///< Selected Dungeons when joined LFG
    selected_dungeons: BTreeSet<u32>,
}
