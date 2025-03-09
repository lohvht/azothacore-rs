use azothacore_common::{az_error, AzError};
use bevy::prelude::{Commands, Resource};
use flagset::flags;
use num::FromPrimitive;
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, ToPrimitive, FromPrimitive, Deserialize, Serialize, PartialEq)]
pub enum ServerProcessType {
    Authserver = 0,
    Worldserver = 1,
}

#[derive(Resource)]
pub struct ThisServerProcess(pub ServerProcessType);

pub fn set_server_process(commands: &mut Commands, typ: ServerProcessType) {
    commands.insert_resource(ThisServerProcess(typ));
}

pub const MAX_CHARACTERS_PER_REALM: u8 = 16;

pub const GUILD_NEWSLOG_MAX_RECORDS: u8 = 25;
pub const GUILD_EVENTLOG_MAX_RECORDS: u8 = 100;
pub const GUILD_BANKLOG_MAX_RECORDS: u8 = 250;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub enum Expansion {
    /// -1, as expansion, is used in CreatureDifficulty.db2 for
    /// auto-updating the levels of creatures having their expansion
    /// set to that value to the current expansion's max leveling level
    LevelCurrent = -1,
    Classic = 0,
    TheBurningCrusade = 1,
    WrathOfTheLichKing = 2,
    Cataclysm = 3,
    MistsOfPandaria = 4,
    WarlordsOfDraenor = 5,
    Legion = 6,
    /// future expansion
    BattleForAzeroth = 7,
}

pub const CURRENT_EXPANSION: Expansion = Expansion::Legion;

#[derive(Debug, serde::Deserialize, serde::Serialize, FromPrimitive, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
#[repr(u8)]
pub enum ItemQuality {
    /// GREY
    Poor = 0,
    /// WHITE
    Normal = 1,
    /// GREEN
    Uncommon = 2,
    /// BLUE
    Rare = 3,
    /// PURPLE
    Epic = 4,
    /// ORANGE
    Legendary = 5,
    /// LIGHT YELLOW
    Artifact = 6,
    /// LIGHT BLUE
    Heirloom = 7,
    /// LIGHT BLUE
    WowToken = 8,
}

impl TryFrom<u8> for ItemQuality {
    type Error = AzError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_u8(value).ok_or(az_error!("unable to convert number '{value}' to ItemQuality"))
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum GmLoginState {
    Disabled = 0,
    Enabled = 1,
    #[default]
    RestoreFromSavedState = 2,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum GmVisibleState {
    Invisible = 0,
    Visible = 1,
    #[default]
    RestoreFromSavedState = 2,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum GmChatState {
    Disabled = 0,
    Enabled = 1,
    #[default]
    RestoreFromSavedState = 2,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum GmWhisperableState {
    Disabled = 0,
    Enabled = 1,
    #[default]
    RestoreFromSavedState = 2,
}

flags! {
    pub enum CleaningFlag: u8 {
        AchievementProgress   = 0x1,
        Skills                = 0x2,
        Spells                = 0x4,
        Talents               = 0x8,
        Queststatus           = 0x10,
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum WardenClientCheckFailAction {
    #[default]
    Disabled,
    Kick,
    Ban,
}

flags! {
    pub enum DungeonFinderOptions: u8 {
        DungeonFinder  = 0b001,
        RaidBrowser    = 0b001,
        SeasonalBosses = 0b001,
        All            = (DungeonFinderOptions::DungeonFinder | DungeonFinderOptions::RaidBrowser | DungeonFinderOptions::SeasonalBosses).bits(),
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum AccountPasswordChangeSecurityPolicy {
    #[default]
    None,
    Email,
    RBAC,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum AutoBroadcastDisplayMethod {
    #[default]
    Announce,
    Notify,
    Both,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum PacketSpoofPolicy {
    #[default]
    LogKick,
    LogOnly,
    LogKickBan,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum PacketSpoofBanMode {
    #[default]
    Account,
    Ip,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
/// Select the preferred format to display information to the player who cannot enter a portal dungeon because when has not met the access requirements:
pub enum DungeonAccessRequirementsPrintMode {
    /// Display only one requirement at a time (BlizzLike, like in the LFG interface)
    #[default]
    Blizzlike,
    /// Display no extra information, only "Requirements  not met"
    NoExtraInfo,
    /// Display detailed requirements, all at once, with clickable links
    DetailedInfo,
}

/// enum Team in TC / AC
#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum FactionID {
    Horde = 67,
    Alliance = 469,
    //TEAM_STEAMWHEEDLE_CARTEL = 169,                       // not used in code
    //TEAM_ALLIANCE_FORCES     = 891,
    //TEAM_HORDE_FORCES        = 892,
    //TEAM_SANCTUARY           = 936,
    //TEAM_OUTLAND             = 980,
    Other = 0, // if ReputationListId > 0 && Flags != FACTION_FLAG_TEAM_HEADER
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{App, Commands, Startup};

    use super::*;

    #[test]
    fn it_sets_server_process() {
        let mut app = App::new();
        assert!(app.world().get_resource::<ThisServerProcess>().is_none());

        app.add_systems(Startup, |mut commands: Commands| {
            set_server_process(&mut commands, ServerProcessType::Worldserver)
        });
        app.update();

        let res = app.world().resource::<ThisServerProcess>();
        assert_eq!(res.0, ServerProcessType::Worldserver);
    }
}
