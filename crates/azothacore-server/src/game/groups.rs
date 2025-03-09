use azothacore_common::{az_error, deref_boilerplate, AzResult};
use bevy::prelude::Component;
use flagset::{flags, FlagSet};

use crate::{
    game::{
        entities::object::object_guid::{HighGuidPlayer, ObjectGuid},
        loot::LootMethod,
    },
    shared::{data_stores::dbc_enums::DifficultyID, shared_defines::ItemQuality},
};

pub mod group_mgr;

flags! {
    /// "enum GroupType" in AC, "enum GroupFlags" in TC
    pub enum GroupFlag: u16 {
        /// GROUP_FLAG_FAKE_RAID
        FakeRaid            = 0x001,
        /// GROUP_FLAG_RAID
        Raid                = 0x002,
        /// GROUP_FLAG_LFG_RESTRICTED
        ///
        /// Script_HasLFGRestrictions()
        LfgRestricted       = 0x004,
        /// GROUP_FLAG_LFG
        Lfg                 = 0x008,
        /// GROUP_FLAG_DESTROYED
        Destroyed           = 0x010,
        /// GROUP_FLAG_ONE_PERSON_PARTY
        ///
        /// Script_IsOnePersonParty()
        OnePersonParty      = 0x020,
        /// GROUP_FLAG_EVERYONE_ASSISTANT
        ///
        /// Script_IsEveryoneAssistant()
        EveryoneAssistant   = 0x040,
        /// GROUP_FLAG_GUILD_GROUP
        GuildGroup          = 0x100,
        /// GROUP_MASK_BGRAID
        Bgraid              = (GroupFlag::FakeRaid | GroupFlag::Raid).bits(),
    }
}

flags! {
    /// "enum GroupMemberFlags" in AC & TC
    pub enum GroupMemberFlags: u16 {
        Assistant   = 0x01,
        Maintank    = 0x02,
        Mainassist  = 0x04,
    }
}

#[derive(Component)]
#[require(
    GroupFlags,
    GroupLootInfo,
    GroupRoundRobinLooterGuid,
    GroupTargetIcons,
    GroupMemberSlots,
    GroupDifficultySettings
)]
pub struct Group {
    /// Group::m_leaderGuid in TC / AC
    leader_guid: ObjectGuid<HighGuidPlayer>,
    /// Group::m_dbStoreId in TC
    ///
    /// Represents the ID used in database (Can be reused by other groups if group was disbanded)
    db_store_id: Option<u32>,
}

#[derive(Component)]
pub struct GroupLootInfo {
    /// Group::m_lootThreshold in TC / AC
    loot_threshold:     ItemQuality,
    /// Group::m_lootMethod in TC / AC
    loot_method:        LootMethod,
    /// Group::m_masterLooterGuid in TC / AC
    master_looter_guid: Option<ObjectGuid<HighGuidPlayer>>,
}

impl Default for GroupLootInfo {
    fn default() -> Self {
        Self {
            loot_threshold:     ItemQuality::Uncommon,
            loot_method:        LootMethod::FreeForAll,
            master_looter_guid: None,
        }
    }
}

/// Group::m_looterGuid in TC / AC
#[derive(Component, Default)]
pub struct GroupRoundRobinLooterGuid(Option<ObjectGuid<HighGuidPlayer>>);

/// Group::m_targetIcons in TC / AC
#[derive(Component, Default)]
pub struct GroupTargetIcons([ObjectGuid; 8]);

deref_boilerplate!(GroupTargetIcons, [ObjectGuid; 8], 0);

/// Group::m_groupFlags in TC, Group::m_groupType in AC
/// Flags usually to denote several multiple states a group can be
/// in, such as raid make everyone assistant etc
#[derive(Component, Default)]
pub struct GroupFlags(FlagSet<GroupFlag>);

deref_boilerplate!(GroupFlags, FlagSet<GroupFlag>, 0);

/// Group::MemberSlot
pub struct GroupMemberSlot {
    player_guid:   ObjectGuid<HighGuidPlayer>,
    // std::string name;
    // uint8       _class;
    flags:         u8,
    roles:         u8,
    ready_checked: bool,
}

pub const MAX_GROUP_SIZE: usize = 5;
pub const MAX_RAID_SIZE: usize = 40;
pub const MAX_RAID_SUBGROUPS: usize = MAX_RAID_SIZE / MAX_GROUP_SIZE;

/// Replaces Group::m_memberSlots and Group::m_subGroupsCounts in TC / AC
///
/// Group::_initRaidSubGroupsCounter() in TC / AC is replaced by `new_raid` / `new_party`
#[derive(Component)]
pub struct GroupMemberSlots {
    slots: Vec<Vec<GroupMemberSlot>>,
}

impl Default for GroupMemberSlots {
    fn default() -> Self {
        Self::new_party()
    }
}

impl GroupMemberSlots {
    pub fn new_party() -> Self {
        Self::_new(true)
    }

    pub fn new_raid() -> Self {
        Self::_new(false)
    }

    fn _new(is_party: bool) -> Self {
        let num_subgroups = if is_party { 1 } else { MAX_RAID_SUBGROUPS };

        let mut slots = Vec::with_capacity(num_subgroups);
        for _ in 0..num_subgroups {
            slots.push(Vec::with_capacity(MAX_GROUP_SIZE));
        }
        Self { slots }
    }

    // fn empty_subgroup(&self) -> Option<usize> {
    //     self.slots.iter().enumerate().find(|(_, sg)| sg.len() < MAX_GROUP_SIZE).map(|i| i.0)
    // }

    pub fn add_member(&mut self, player_guid: ObjectGuid<HighGuidPlayer>, member_flags: u8, subgroup: usize, roles: u8) -> AzResult<()> {
        let slots = &mut self.slots[subgroup];
        if subgroup < MAX_RAID_SUBGROUPS - 1 {
            return Err(az_error!("subgroup {subgroup} is not valid"));
        }
        if slots.len() > MAX_GROUP_SIZE {
            return Err(az_error!(
                "subgroup {subgroup} already has more than {MAX_GROUP_SIZE} members, has {}",
                slots.len()
            ));
        }
        slots.push(GroupMemberSlot {
            player_guid,
            flags: member_flags,
            roles,
            ready_checked: false,
        });
        Ok(())
    }
}

#[derive(Component)]
pub struct GroupDifficultySettings {
    /// Group::m_dungeonDifficulty in TC/AC
    dungeon:     DifficultyID,
    /// Group::m_raidDifficulty in TC/AC
    raid:        DifficultyID,
    /// Group::m_legacyRaidDifficulty in TC
    legacy_raid: DifficultyID,
}

impl Default for GroupDifficultySettings {
    fn default() -> Self {
        Self {
            dungeon:     DifficultyID::Normal,
            raid:        DifficultyID::NormalRaid,
            legacy_raid: DifficultyID::_10N,
        }
    }
}

// /// NOTE: TC
// Group::Group() : m_leaderGuid(), m_leaderName(""), m_groupFlags(GROUP_FLAG_NONE), m_groupCategory(GROUP_CATEGORY_HOME),
// m_dungeonDifficulty(DIFFICULTY_NORMAL), m_raidDifficulty(DIFFICULTY_NORMAL_RAID), m_legacyRaidDifficulty(DIFFICULTY_10_N),
// m_bgGroup(nullptr), m_bfGroup(nullptr), m_lootMethod(FREE_FOR_ALL), m_lootThreshold(ITEM_QUALITY_UNCOMMON), m_looterGuid(),
// m_masterLooterGuid(), m_subGroupsCounts(nullptr), m_guid(), m_maxEnchantingLevel(0), m_dbStoreId(0),
// m_readyCheckStarted(false), m_readyCheckTimer(0), m_activeMarkers(0)
// {
//     for (uint8 i = 0; i < TARGET_ICONS_COUNT; ++i)
//         m_targetIcons[i].Clear();

//     for (uint8 i = 0; i < RAID_MARKERS_COUNT; ++i)
//         m_markers[i] = nullptr;
// }

// GroupRefManager     m_memberMgr;                        GroupRefMgr         m_memberMgr;
// InvitesList         m_invitees;                         InvitesList         m_invitees;
// std::string         m_leaderName;                       std::string         m_leaderName;
// GroupCategory       m_groupCategory; // NOTE: TC Only - // group category => usually related to `GroupCategory` + `GroupType`` of a player. If change in a group's GUID of the same category, an update sequence number is reset to 1. m_groupCategory is only set to instance on ConvertToLFG() and Battleground / battlefield groups.
// Battleground*       m_bgGroup;                          Battlefield*        m_bfGroup;
// Battlefield*        m_bfGroup;                          Battleground*       m_bgGroup;
// LootMethod          m_lootMethod;                       LootMethod          m_lootMethod;
// Rolls               RollId;                             Rolls               RollId;
// BoundInstancesMap   m_boundInstances;
// ObjectGuid          m_guid;                             ObjectGuid          m_guid;
// uint32              m_counter;                      // NOTE: Seems like AC only, used only in SMSG_GROUP_LIST.
// uint32              m_maxEnchantingLevel;               uint32              m_maxEnchantingLevel;
// bool                m_readyCheckStarted; // Ready Check
// int32               m_readyCheckTimer; // Ready Check
// uint8               m_lfgGroupFlags; // NOTE: AC Only - TODO: Find out what is this for
// // Raid markers
// std::array<std::unique_ptr<RaidMarker>, RAID_MARKERS_COUNT> m_markers; // NOTE: TC Only - this should be for raid markers. AC is on 3.3.5 which doesnt support raid markers. Previously rogues used to use smoke bombs
// uint32              m_activeMarkers; // mask containing the set raid markets

// uint32 _difficultyChangePreventionTime; // NOTE: AC Only // Xinef: change difficulty prevention
// DifficultyPreventionChangeType _difficultyChangePreventionType; // NOTE: AC Only // Xinef: change difficulty prevention

// /// NOTE: AC

// Group::Group() : m_leaderName(""), m_groupType(GROUPTYPE_NORMAL),
//     m_dungeonDifficulty(DUNGEON_DIFFICULTY_NORMAL), m_raidDifficulty(RAID_DIFFICULTY_10MAN_NORMAL),
//     m_bfGroup(nullptr), m_bgGroup(nullptr), m_lootMethod(FREE_FOR_ALL), m_lootThreshold(ITEM_QUALITY_UNCOMMON),
//     m_subGroupsCounts(nullptr), m_counter(0), m_maxEnchantingLevel(0), _difficultyChangePreventionTime(0),
//     _difficultyChangePreventionType(DIFFICULTY_PREVENTION_CHANGE_NONE)
// {
//     sScriptMgr->OnConstructGroup(this);
// }
