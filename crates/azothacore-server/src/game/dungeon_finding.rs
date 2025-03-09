use azothacore_common::{
    az_error,
    bevy_app::{query_not_found_result, QueryOrNewSingleMut, ToFromEntity},
    configuration::ConfigMgr,
    AzResult,
};
use bevy::{
    ecs::system::SystemParam,
    prelude::{Commands, Query, Res},
};
use lfg::LfgState;
use lfg_group_data::{KicksLeftConfig, LfgGroupData};
use lfg_player_data::LfgPlayerData;
use num_traits::FromPrimitive;

use super::entities::object::object_guid::HighGuidPlayer;
use crate::game::entities::object::object_guid::{HighGuidParty, ObjectGuid};

pub mod lfg;
pub mod lfg_group_data;
pub mod lfg_player_data;

/// Reserve this for read only methods for LFGMgr
/// TODO: Implement me!
pub trait LFGMgrTrait {}

#[derive(SystemParam)]
pub struct LFGMgr<'w, 's, C: KicksLeftConfig> {
    commands:      Commands<'w, 's>,
    config:        Res<'w, ConfigMgr<C>>,
    /// LFGMgr::GroupsStore in TC / AC
    ///
    ///< Group data
    groups_store:  Query<'w, 's, &'static mut LfgGroupData>,
    /// LFGMgr::PlayersStore in TC / AC
    ///
    ///< Player data
    players_store: Query<'w, 's, &'static mut LfgPlayerData>,
}

impl<C: KicksLeftConfig> LFGMgrTrait for LFGMgr<'_, '_, C> {}

impl<C: KicksLeftConfig> LFGMgr<'_, '_, C> {
    /// LFGMgr::_LoadFromDB in TC / AC
    ///
    /// Loads the proper LfgGroupData, don't add to world via commands here yet.
    pub fn new_lfg_group_data_from_dungeon_and_state(&self, dungeon_id: u32, state: u8) -> AzResult<LfgGroupData> {
        if dungeon_id == 0 {
            return Err(az_error!("dungeon_id is zero; dungeon_id={dungeon_id}"));
        }
        let Some(state) = LfgState::from_u8(state) else {
            return Err(az_error!("state is invalid; state={state}"));
        };
        let mut group_data = LfgGroupData::new(&**self.config);
        group_data.dungeon_id = dungeon_id;
        if matches!(state, LfgState::Dungeon) || matches!(state, LfgState::FinishedDungeon) {
            group_data.set_state(state);
        }
        Ok(group_data)
    }

    // /// LFGMgr::SetupGroupMember in TC / AC
    // // TODO: COMPLETE ME!
    // pub fn setup_group_member(&self, guid: ObjectGuid<HighGuidPlayer>, gguid: ObjectGuid<HighGuidParty>)
    // {
    //     LfgDungeonSet dungeons;
    //     dungeons.insert(GetDungeon(gguid));
    //     SetSelectedDungeons(guid, dungeons);
    //     SetState(guid, GetState(gguid));
    //     SetGroup(guid, gguid);
    //     AddPlayerToGroup(gguid, guid);
    // }

    fn get_or_init_group_lfg_data_mut(&mut self, guid: ObjectGuid<HighGuidParty>) -> AzResult<QueryOrNewSingleMut<LfgGroupData>> {
        let e = guid.to_entity();
        query_not_found_result(self.groups_store.get_mut(e))
            .map(|v| match v {
                None => QueryOrNewSingleMut::New(Some(LfgGroupData::new(&**self.config)), self.commands.entity(e)),
                Some(v) => QueryOrNewSingleMut::Existing(v),
            })
            .map_err(|e| e.context("cannot retrieve group data for {guid}"))
    }

    // fn get_or_init_player_lfg_data_mut(
    //     &mut self,
    //     guid: ObjectGuid<HighGuidPlayer>,
    // ) -> AzResult<QueryOrNewSingleMut<LfgPlayerData>> {
    //     let e = guid.to_entity();
    //     query_not_found_result(self.players_store.get_mut(e))
    //         .map(|v| match v {
    //             None => QueryOrNewSingleMut::New(Some(LfgGroupData::new(&**self.config)), self.commands.entity(e)),
    //             Some(v) => QueryOrNewSingleMut::Existing(v),
    //         })
    //         .map_err(|e| e.context("cannot retrieve group data for {guid}"))
    // }
}
