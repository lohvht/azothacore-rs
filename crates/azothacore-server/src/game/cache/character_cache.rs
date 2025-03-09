use bevy::prelude::Resource;

use crate::game::entities::object::object_guid::{HighGuidPlayer, ObjectGuid};

#[derive(Resource)]
pub struct CharacterCache {}

impl CharacterCache {
    /// CharacterCache::GetCharacterNameByGuid in AC, ObjectMgr::GetPlayerNameByGUID in TC
    pub fn get_character_name_by_guid(&self, _guid: ObjectGuid<HighGuidPlayer>) -> Option<String> {
        todo!()
    }
}
