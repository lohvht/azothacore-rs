use std::{
    mem,
    net::IpAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use azothacore_common::{
    bounded_nums::{LowerBoundedNum, LowerBoundedNumMissingDefault, RangedBoundedNum, UpperBoundedNum},
    collision::management::vmap_mgr2::VmapConfig,
    configuration::{from_env_toml, Config, DataDirConfig, DatabaseInfo, DbUpdates, LogAppender, LogFlags, LogLevel, LogLoggerConfig},
    durationb,
    durationb_days,
    durationb_hours,
    durationb_mins,
    durationb_ms,
    durationb_s,
    f32b,
    log::LoggingConfig,
    AccountTypes,
    AzResult,
    Locale,
    BASE_DIR,
};
use flagset::FlagSet;
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;
use tracing::error;

use crate::{
    game::{
        battlegrounds::BattlegroundQueueInvitationType,
        entities::{
            object::object_defines::{
                CONTACT_DISTANCE,
                DEFAULT_VISIBILITY_BGARENAS,
                DEFAULT_VISIBILITY_DISTANCE,
                DEFAULT_VISIBILITY_INSTANCE,
                MAX_VISIBILITY_DISTANCE,
                NOMINAL_MELEE_RANGE,
            },
            player::MAX_MONEY_AMOUNT,
        },
        globals::object_mgr::{MAX_CHARTER_NAME, MAX_PET_NAME, MAX_PLAYER_NAME},
        grid::{
            grid_defines::{MIN_GRID_DELAY, MIN_MAP_UPDATE_DELAY},
            DEFAULT_VISIBILITY_NOTIFY_PERIOD,
        },
        world::{
            ArenaQueueAnnouncerDetail,
            CharDeleteMethod,
            CharacterCreateClassDisabled,
            CharacterCreateFactionDisabled,
            CharacterCreateRaceDisabled,
            ChatStrictLinkCheckingKick,
            ChatStrictLinkCheckingSeverity,
            GroupVisibilityMode,
            ItemDeleteMethod,
            PvPTokenMapAllowType,
            RealmZone,
            SkipCinematics,
            StrictName,
            TalentsInspectingMode,
        },
    },
    shared::{
        data_stores::dbc_enums::{LEVEL_LIMIT_MAX, LEVEL_LIMIT_MAX_DEFAULT},
        realms::{realm_list::RealmListConfig, RealmType},
        shared_defines::{
            AccountPasswordChangeSecurityPolicy,
            AutoBroadcastDisplayMethod,
            CleaningFlag,
            DungeonAccessRequirementsPrintMode,
            DungeonFinderOptions,
            Expansion,
            GmChatState,
            GmLoginState,
            GmVisibleState,
            GmWhisperableState,
            ItemQuality,
            PacketSpoofBanMode,
            PacketSpoofPolicy,
            WardenClientCheckFailAction,
            CURRENT_EXPANSION,
            GUILD_BANKLOG_MAX_RECORDS,
            GUILD_EVENTLOG_MAX_RECORDS,
            GUILD_NEWSLOG_MAX_RECORDS,
            MAX_CHARACTERS_PER_REALM,
        },
    },
};

pub fn default_worldserver_log_appenders() -> Vec<LogAppender> {
    // use LogConsoleColours::*;
    use LogFlags::*;
    use LogLevel::*;
    vec![
        LogAppender::Console {
            name:      String::from("Console"),
            min_level: Info,
            max_level: Error,
            flags:     AddLogLevel | AddLogFilter | TruncateFile | BackupBeforeOverwrite,
            // colours: vec![
            //     (Fatal, Red),
            //     (Error, Lred),
            //     (Warning, Brown),
            //     (Info, Green),
            //     (Debug, Cyan),
            //     (Trace, Magenta),
            // ],
        },
        LogAppender::File {
            name:      String::from("Server"),
            min_level: Warning,
            max_level: Error,
            flags:     AddLogLevel | AddLogFilter | TruncateFile | BackupBeforeOverwrite | AddLogTimestamps,
            file:      String::from("Server.log"),
        },
        LogAppender::File {
            name:      String::from("GM"),
            min_level: Warning,
            max_level: Error,
            flags:     AddLogTimestamps | AddLogLevel | AddLogFilter | AppendFileTimestamps | TruncateFile | BackupBeforeOverwrite,
            file:      String::from("gm.log"),
        },
        LogAppender::File {
            name:      String::from("DBErrors"),
            min_level: Warning,
            max_level: Error,
            flags:     TruncateFile | BackupBeforeOverwrite,
            file:      String::from("DBErrors.log"),
        },
    ]
}

pub fn default_worldserver_log_configs() -> Vec<LogLoggerConfig> {
    use LogLevel::*;
    vec![
        LogLoggerConfig {
            name:      String::from("root"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("Server")],
        },
        LogLoggerConfig {
            name:      String::from("module"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("Server")],
        },
        LogLoggerConfig {
            name:      String::from("commands::gm"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("GM")],
        },
        LogLoggerConfig {
            name:      String::from("diff"),
            min_level: Warning,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("Server")],
        },
        LogLoggerConfig {
            name:      String::from("mmaps"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Server")],
        },
        LogLoggerConfig {
            name:      String::from("server"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("Server")],
        },
        LogLoggerConfig {
            name:      String::from("sql::sql"),
            min_level: Warning,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("DBErrors")],
        },
        LogLoggerConfig {
            name:      String::from("sql"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("Server")],
        },
        LogLoggerConfig {
            name:      String::from("time::update"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("Server")],
        },
    ]
}

type GtZeroOrOneF32 = LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(1.0) }>;

structstruck::strike! {
#[strikethrough[serde_inline_default]]
#[strikethrough[derive(DefaultFromSerde, Deserialize, Serialize, Clone, Debug, PartialEq)]]
pub struct WorldConfig {
    #[serde_inline_default(DatabaseInfo::default_with_info("azcore_auth"))] pub LoginDatabaseInfo: DatabaseInfo,
    #[serde_inline_default(DatabaseInfo::default_with_info("azcore_world"))] pub WorldDatabaseInfo: DatabaseInfo,
    #[serde_inline_default(DatabaseInfo::default_with_info("azcore_characters"))] pub CharacterDatabaseInfo: DatabaseInfo,
    #[serde_inline_default(DatabaseInfo::default_with_info("azcore_hotfixes"))] pub HotfixDatabaseInfo: DatabaseInfo,
    #[serde_inline_default(DbUpdates{EnableDatabases: None.into(), ..Default::default() })] pub Updates: DbUpdates,
    /// Default Message of the Day, displayed at login.
    /// This value is the fallback for when no other Motd can be found in the `azcore_auth.motd` table.
    #[serde_inline_default("Welcome to a Azothacore Server.".into())] Motd: String,
    #[serde_inline_default(format!("{BASE_DIR}/data").into())] pub DataDir: PathBuf,
    #[serde_inline_default(format!("{BASE_DIR}/logs").into())] pub LogsDir: PathBuf,
    #[serde(default="default_worldserver_log_appenders")] pub Appender: Vec<LogAppender>,
    #[serde(default="default_worldserver_log_configs")] pub Logger: Vec<LogLoggerConfig>,
    /// Time between realm list updates.
    #[serde(default)] pub RealmsStateUpdateDelay: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(10) }>,
    #[serde_inline_default(1)] pub RealmID: u32,
    #[serde_inline_default("0.0.0.0".parse().unwrap())] pub BindIP: IpAddr,
    #[serde(default)] pub ClientCacheVersion: u32,
    #[serde(default)] pub HotfixCacheVersion: u32,
    #[serde_inline_default(Locale::enUS)] pub DBCLocale: Locale,
    /// Maximum number of players in the world. Excluding Mods, GMs and Admins.
    /// If you want to block players and only allow Mods, GMs or Admins to join the
    /// server, use the DB field "auth.realmlist.allowedSecurityLevel".
    ///
    /// Put zero to disable a limit.
    #[serde_inline_default(0usize)] pub PlayerLimit: usize,
    #[serde(default)] pub PlayerStart: pub struct WorldConfigPlayerStart {
        /// String to be displayed at first login of newly created characters - default empty for disabled
        #[serde(default)] pub String: String,
        #[serde(default)] pub AllReputation: bool,
        #[serde(default)] pub CustomSpells: bool,
        #[serde(default)] pub MapsExplored: bool,
    },
    /// Display core version (`.server info`) on login.
    ///
    /// Server.LoginInfo OR CONFIG_ENABLE_SINFO_LOGIN in TC/AC
    #[serde(default)] pub SendServerInfoOnLogin: bool,
    /// Server rates - Read all rates from the config file
    #[serde(default)] pub Rate: pub struct WorldConfigRate {
        #[serde(default)] pub Health: GtZeroOrOneF32,
        #[serde(default)] pub Mana: GtZeroOrOneF32,
        /// Multiplier to configure health and power increase or decrease.
        #[serde(default)] pub Power: pub struct WorldConfigRatePower {
            #[serde(default)] pub RageGain: GtZeroOrOneF32,
            #[serde(default)] pub RageLoss: GtZeroOrOneF32,
            #[serde(default)] pub RunicPowerGain: GtZeroOrOneF32,
            #[serde(default)] pub RunicPowerLoss: GtZeroOrOneF32,
            #[serde(default)] pub Focus: GtZeroOrOneF32,
            #[serde(default)] pub Energy: GtZeroOrOneF32,
            #[serde(default)] pub ComboPointLoss: GtZeroOrOneF32,
            #[serde(default)] pub SoulShardsLoss: GtZeroOrOneF32,
            #[serde(default)] pub LunarPowerLoss: GtZeroOrOneF32,
            #[serde(default)] pub HolyPowerLoss: GtZeroOrOneF32,
            #[serde(default)] pub MaelstromLoss: GtZeroOrOneF32,
            #[serde(default)] pub ChiLoss: GtZeroOrOneF32,
            #[serde(default)] pub InsanityLoss: GtZeroOrOneF32,
            #[serde(default)] pub ArcaneChargesLoss: GtZeroOrOneF32,
            #[serde(default)] pub FuryLoss: GtZeroOrOneF32,
            #[serde(default)] pub PainLoss: GtZeroOrOneF32,
        },
        /// Multiplier for skill discovery.
        #[serde(default)] pub SkillDiscovery: GtZeroOrOneF32,
        /// Drop rates for money and items based on quality.
        #[serde(default)] pub DropItem: pub struct WorldConfigRateDropItem {
            #[serde(default)] pub Poor: GtZeroOrOneF32,
            #[serde(default)] pub Normal: GtZeroOrOneF32,
            #[serde(default)] pub Uncommon: GtZeroOrOneF32,
            #[serde(default)] pub Rare: GtZeroOrOneF32,
            #[serde(default)] pub Epic: GtZeroOrOneF32,
            #[serde(default)] pub Legendary: GtZeroOrOneF32,
            #[serde(default)] pub Artifact: GtZeroOrOneF32,
            #[serde(default)] pub Referenced: GtZeroOrOneF32,
            #[serde(default)] pub ReferencedAmount: GtZeroOrOneF32,
            #[serde(default)] pub GroupAmount: GtZeroOrOneF32,
        },
        #[serde(default)] pub DropMoney: GtZeroOrOneF32,
        #[serde(default)] pub BuyValueItem: pub struct WorldConfigRateValueItem {
            #[serde(default)] pub Poor: GtZeroOrOneF32,
            #[serde(default)] pub Normal: GtZeroOrOneF32,
            #[serde(default)] pub Uncommon: GtZeroOrOneF32,
            #[serde(default)] pub Rare: GtZeroOrOneF32,
            #[serde(default)] pub Epic: GtZeroOrOneF32,
            #[serde(default)] pub Legendary: GtZeroOrOneF32,
            #[serde(default)] pub Artifact: GtZeroOrOneF32,
            #[serde(default)] pub Heirloom: GtZeroOrOneF32,
        },
        #[serde(default)] pub SellValueItem: WorldConfigRateValueItem,
        /// Experience rates
        #[serde(default)] pub XP: pub struct WorldConfigRateExperience {
            #[serde(default)] pub Kill: GtZeroOrOneF32,
            #[serde(default)] pub Quest: GtZeroOrOneF32,
            #[serde(default)] pub QuestDF: GtZeroOrOneF32,
            #[serde(default)] pub Explore: GtZeroOrOneF32,
            #[serde(default)] pub Pet: GtZeroOrOneF32,
            #[serde(default)] pub PetLevelXP: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(0.05) }>,
            /// Experience rate for honorable kills in Alterac Valley (AV). Not affected by Rate.XP.Kill. Only works if Battleground.GiveXPForKills = true
            #[serde(default)] pub BattlegroundKillAV: GtZeroOrOneF32,
            /// Experience rate for honorable kills in Warsong Gulch (WSG). Not affected by Rate.XP.Kill. Only works if Battleground.GiveXPForKills = true
            #[serde(default)] pub BattlegroundKillWSG: GtZeroOrOneF32,
            /// Experience rate for honorable kills in Arathi Basin (AB). Not affected by Rate.XP.Kill. Only works if Battleground.GiveXPForKills = true
            #[serde(default)] pub BattlegroundKillAB: GtZeroOrOneF32,
            /// Experience rate for honorable kills in Eye of the Storm (EOTS). Not affected by Rate.XP.Kill. Only works if Battleground.GiveXPForKills = true
            #[serde(default)] pub BattlegroundKillEOTS: GtZeroOrOneF32,
            /// Experience rate for honorable kills in Strand of the Ancients (SOTA). Not affected by Rate.XP.Kill. Only works if Battleground.GiveXPForKills = true
            #[serde(default)] pub BattlegroundKillSOTA: GtZeroOrOneF32,
            /// Experience rate for honorable kills in Isle of Conquest (IC). Not affected by Rate.XP.Kill. Only works if Battleground.GiveXPForKills = true
            #[serde(default)] pub BattlegroundKillIC: GtZeroOrOneF32,
        },
        #[serde(default)] pub RepairCost: GtZeroOrOneF32,
        #[serde(default)] pub Reputation: pub struct WorldConfigRateReputation {
            /// Reputation gain rate
            #[serde(default)] pub Gain: GtZeroOrOneF32,
            /// Reputation gain from killing low level (grey) creatures.
            #[serde(default)] pub LowLevelKill: GtZeroOrOneF32,
            /// Reputation gain rate from quests
            #[serde(default)] pub LowLevelQuest: GtZeroOrOneF32,
            /// Reputation bonus rate for recruit-a-friend.
            #[serde(default)] pub RecruitAFriendBonus: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(0.1) }>,
        },
        #[serde(default)] pub Creature: pub struct WorldConfigRateCreature {
            #[serde(default)] pub Aggro: GtZeroOrOneF32,
            /// Mulitplier for creature melee damage
            #[serde(default)] pub Damage: pub struct WorldConfigRateCreatureStatsRate {
                #[serde(default)] pub Normal: GtZeroOrOneF32,
                #[serde(default)] pub Elite: GtZeroOrOneF32,
                #[serde(default)] pub Rare: GtZeroOrOneF32,
                #[serde(default)] pub RareElite: GtZeroOrOneF32,
                #[serde(default)] pub WORLDBOSS: GtZeroOrOneF32,
            },
            /// Mulitplier for creature health.
            #[serde(default)] pub HP: WorldConfigRateCreatureStatsRate,
            /// Mulitplier for creature spell damage.
            #[serde(default)] pub SpellDamage: WorldConfigRateCreatureStatsRate,
        },
        /// Resting points grow rates.
        #[serde(default)] pub Rest: pub struct WorldConfigRateRest {
            #[serde(default)] pub InGame: GtZeroOrOneF32,
            #[serde(default)] pub OfflineInTavernOrCity: GtZeroOrOneF32,
            #[serde(default)] pub OfflineInWilderness: GtZeroOrOneF32,
        },
        /// Damage after fall rate.
        #[serde(default)] FallDamage: GtZeroOrOneF32,
        #[serde(default)] pub Auction: pub struct WorldConfigRateAuction {
            #[serde(default)] pub Time: GtZeroOrOneF32,
            #[serde(default)] pub Deposit: GtZeroOrOneF32,
            #[serde(default)] pub Cut: GtZeroOrOneF32,
        },
        #[serde(default)] pub Honour: GtZeroOrOneF32,
        /// Multiplier for the rate between global raid/heroic instance resets
        /// (dbc value). Higher value increases the time between resets,
        /// lower value lowers the time, you need clean instance_reset in
        /// characters db in order to let new values work.
        #[serde(default)] pub InstanceResetTime: GtZeroOrOneF32,
        /// Movement speed rate.
        #[serde(default)] pub MoveSpeed: GtZeroOrOneF32,
        #[serde(default)] pub CorpseDecayLooted: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(0.5) }>,
        #[serde(default)] pub MissChanceMultiplier: pub struct WorldConfigRateMissChanceMultiplier {
            #[serde(default)] pub TargetCreature: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(11.00) }>,
            #[serde(default)] pub TargetPlayer: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(7.00) }>,
            #[serde(default)] pub OnlyAffectsPlayer: bool,
        },
        #[serde(default)] pub Quest: pub struct WorldConfigRateQuest {
            /// Rate.Quest.Money.Reward in TC / Rate.RewardQuestMoney in AC
            #[serde(default)] pub MoneyReward: LowerBoundedNumMissingDefault<f32, { f32b!(0.0) }, { f32b!(0.0) }, { f32b!(1.0) }>,
            /// Rate.Quest.Money.Max.Level.Reward in TC / Rate.RewardBonusMoney in AC
            #[serde(default)] pub MaxLevelReward: LowerBoundedNumMissingDefault<f32, { f32b!(0.0) }, { f32b!(0.0) }, { f32b!(1.0) }>,
        },
    },
    /// Max distance from movement target point (+moving unit size) and targeted
    /// object (+size) after that new target movement point calculated.
    /// valid between [CONTACT_DISTANCE] and [NOMINAL_MELEE_RANGE]. defaults to 1.5
    ///
    /// [CONTACT_DISTANCE] - Minimum, Contact Range, More sensitive reaction to target movement
    /// [NOMINAL_MELEE_RANGE] - Maximum, Melee attack range, Less CPU usage
    #[serde(default)] pub TargetPosRecalculateRange: RangedBoundedNum<f32, { f32b!(CONTACT_DISTANCE) }, { f32b!(NOMINAL_MELEE_RANGE) }, { f32b!(1.5) }>,
    #[serde(default)] pub DurabilityLoss: pub struct WorldConfigDurabilityLoss {
        /// Durability loss percentage on death.
        #[serde(default)] pub OnDeath: RangedBoundedNum<f32, { f32b!(0.0) }, { f32b!(1.0) }, { f32b!(0.1) }>,
        /// Chance to lose durability on one equipped item from damage
        ///
        /// default - 100/0.5 = 200, Each 200 damage one equipped item will use durability
        #[serde(default)] pub ChanceDamage: LowerBoundedNumMissingDefault<f32, { f32b!(0.0) }, { f32b!(0.0) }, { f32b!(0.5) }>,
        /// Chance to lose durability on one equipped armor item when absorbing damage.
        ///
        /// default - 100/0.5 = 200, Each 200 absorbed damage one equipped item will lose durability
        #[serde(default)] pub ChanceAbsorb: LowerBoundedNumMissingDefault<f32, { f32b!(0.0) }, { f32b!(0.0) }, { f32b!(0.5) }>,
        /// Chance to lose durability on main weapon when parrying attacks.
        ///
        /// default - 100/0.05 = 2000, Each 2000 parried damage the main weapon will lose durability
        #[serde(default)] pub ChanceParry: LowerBoundedNumMissingDefault<f32, { f32b!(0.0) }, { f32b!(0.0) }, { f32b!(0.05) }>,
        /// Chance to lose durability on shield when blocking attacks.
        ///
        /// default - 100/0.05 = 2000, Each 2000 blocked damage the shield will lose durability
        #[serde(default)] pub ChanceBlock: LowerBoundedNumMissingDefault<f32, { f32b!(0.0) }, { f32b!(0.0) }, { f32b!(0.05) }>,
        /// Durability loss on death during PvP.
        #[serde(default)] pub InPvP: bool,
    },
    /// Configure the use of the addon channel through the server (some client side
    /// addons will not work correctly with disabled addon channel)
    #[serde_inline_default(true)] pub AddonChannel: bool,
    /// Clean out deprecated achievements, skills, spells and talents from the db.
    #[serde(default)] pub CleanCharacterDB: bool,
    /// Store custom chat channel settings like password, automatic ownership handout
    /// or ban list in the database. Needs to be enabled to save custom
    /// world/trade/etc. channels that have automatic ownership handout disabled.
    /// (`.channel set ownership $channel off`)
    #[serde(default)] pub PreserveCustomChannels: bool,
    /// Unload grids to save memory. Can be disabled if enough memory is available
    /// to speed up moving players to new grids.
    #[serde_inline_default(true)] pub GridUnload: bool,
    /// Load all grids for base maps upon load. Requires GridUnload to be 0.
    /// This will take around 5GB of ram upon server load, and will take some time
    /// to initially load the server.
    #[serde(default)] pub BaseMapLoadAllGrids: bool,
    /// Load all grids for instance maps upon load. Requires GridUnload to be 0.
    /// Upon loading an instance map, all creatures/objects in the map will be pre-loaded
    #[serde(default)] pub InstanceMapLoadAllGrids: bool,
    #[serde(default)] pub PlayerSave: pub struct WorldConfigPlayerSave {
        /// Save player stats only on logout.
        #[serde_inline_default(true)] pub SaveOnlyOnLogout: bool,
        #[serde(default)] pub StatsInterval: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(15 * 60) } >,
        /// Minimum level for saving character stats in the database for external usage. default 0 disables saving
        #[serde(default)] pub StatsMinLevel: RangedBoundedNum<u32, 0, { LEVEL_LIMIT_MAX as i128 }, 0>,
    },
    #[serde_inline_default(true)] pub CloseIdleConnections: bool,
    #[serde(default)] pub AllowTwoSides: pub struct WorldConfigAllowTwoSides {
        #[serde_inline_default(true)] pub Accounts: bool,
        #[serde(default)] pub InteractionCalendar: bool,
        #[serde(default)] pub InteractionChat: bool,
        #[serde(default)] pub InteractionChannel: bool,
        #[serde(default)] pub InteractionGroup: bool,
        #[serde(default)] pub InteractionGuild: bool,
        #[serde(default)] pub InteractionArena: bool,
        #[serde(default)] pub InteractionAuction: bool,
        #[serde(default)] pub InteractionMail: bool,
        #[serde(default)] pub InteractionEmote: bool,
        #[serde(default)] pub WhoList: bool,
        #[serde(default)] pub AddFriend: bool,
        #[serde(default)] pub Trade: bool,
    },
    #[serde(default)] pub StrictNames: pub struct WorldConfigStrictNames {
        #[serde_inline_default(true)] pub Reserved: bool,
        #[serde_inline_default(true)] pub Profanity: bool,
    },
    #[serde(default)] pub AllFlightPaths: bool,
    #[serde(default)] pub InstantFlightPaths: bool,
    #[serde_inline_default(true)] pub AllowPlayerCommands: bool,
    #[serde(default)] pub Instance: pub struct WorldConfigInstance {
        #[serde(default)] pub IgnoreLevel: bool,
        #[serde(default)] pub IgnoreRaid: bool,
        #[serde(default)] pub GMSummonPlayer: bool,
        #[serde(default)] pub SharedNormalHeroicId: bool,
        #[serde(default)] pub ResetTimeHour: UpperBoundedNum<u32, 23, 4>,
        #[serde_inline_default(1135814400)] pub ResetTimeRelativeTimestamp: u64, // TODO: convert to time
        #[serde(default)] pub UnloadDelay: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(30) }>,
    },
    #[serde_inline_default(true)] pub CastUnstuck: bool,
    #[serde(default)] pub GM: pub struct WorldConfigGM {
        #[serde(default)] pub LoginState: GmLoginState,
        #[serde(default)] pub Visible: GmVisibleState,
        #[serde(default)] pub Chat: GmChatState,
        #[serde(default)] pub WhisperingTo: GmWhisperableState,
        #[serde(default)] pub FreezeAuraDuration: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(0) }>,
        #[serde_inline_default(AccountTypes::SecAdministrator)] pub LevelInGMList: AccountTypes,
        #[serde_inline_default(AccountTypes::SecAdministrator)] pub LevelInWhoList: AccountTypes,
        #[serde(default)] pub StartLevel: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>,
        #[serde(default)] pub AllowInvite: bool,
        #[serde(default)] pub AllowFriend: bool,
        #[serde(default)] pub LowerSecurity: bool,
        #[serde(default)] pub ForceShutdownThreshold: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(30) } >,
    },
    /// Max distance to creature for group member to get experience at creature death.
    #[serde(default)] pub MaxGroupXPDistance: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(74.0) }>,
    /// Max distance between character and and group to gain the Recruit-A-Friend
    /// XP multiplier.
    #[serde(default)] pub MaxRecruitAFriendBonusDistance: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(100.0) }>,
    /// The maximum distance in yards that a "monster" creature can see
    /// regardless of level difference (through CreatureAI::IsVisible).
    /// Increases CONFIG_SIGHT_MONSTER to 50 yards. Used to be 20 yards.
    #[serde(default)] pub MonsterSight: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(50.0) }>,
    #[serde(default)] pub Compression: RangedBoundedNum<u32, 1, 9, 1>,
    /// Determines the character clean flags that remain set after cleanups.
    /// This is a bitmask value, check the enum value [crate::shared::shared_defines::CleaningFlag] for more
    /// information.
    #[serde(default)] pub PersistentCharacterCleanFlags: FlagSet<CleaningFlag>,
    #[serde(default)] pub Auction: pub struct WorldConfigAuction {
        /// Sets the minimum time a single player character can perform a getall scan.
        /// The value is only held in memory so a server restart will clear it.
        /// Setting this to zero, will disable GetAll functions completely.
        #[serde(default)] pub GetAllScanDelay: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(15) } >,
        /// Sets the minimum time that the client must wait between
        /// auction search operations. This can be increased if somehow Auction House activity is causing
        /// too much load.
        #[serde(default)] pub SearchDelay: RangedBoundedNum<Duration, { durationb_ms!(100) }, { durationb_ms!(10000) }, { durationb_ms!(300) } >,
    },
    /// Level requirements for several functions, such as chatting in channels, whispers, trade, send mail etc.
    #[serde(default)] pub LevelReq: pub struct WorldConfigLevelReq {
        #[serde(default)] pub ChatChannel: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>,
        #[serde(default)] pub ChatWhisper: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>,
        #[serde(default)] pub ChatEmote: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>,
        #[serde(default)] pub ChatSay: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>,
        #[serde(default)] pub ChatYell: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>,
        #[serde(default)] pub Party: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>,
        #[serde(default)] pub Trade: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>,
        #[serde(default)] pub Auction: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>,
        #[serde(default)] pub Mail: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>,
    },
    /// Time that needs to pass before the customs chat channels get
    /// cleaned up from the database. Only channels with ownership handout enabled
    /// (default behavior) will be cleaned.
    #[serde(default)] pub PreserveCustomChannelDuration: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(0) }>,
    /// Allows to skip queue after being disconnected for a given number of seconds.
    #[serde(default)] pub DisconnectToleranceInterval: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_days!(14) }>,
    /// Time (in milliseconds) grid clean up delay.
    #[serde(default)] pub GridCleanUpDelay: LowerBoundedNumMissingDefault<Duration, { durationb!(MIN_GRID_DELAY) }, { durationb!(MIN_GRID_DELAY) }, { durationb_mins!(5) }>,
    /// Time for player save interval.
    #[serde(default)] pub PlayerSaveInterval: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(15) }>,
    /// Time for map update interval.
    #[serde(default)] pub MapUpdateInterval:  LowerBoundedNumMissingDefault<Duration, { durationb!(MIN_MAP_UPDATE_DELAY) }, { durationb!(MIN_MAP_UPDATE_DELAY) }, { durationb_ms!(100) }>,
    /// Time for weather update interval.
    #[serde(default)] pub ChangeWeatherInterval: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(10) }>,
    /// TCP port to reach the world server.
    #[serde_inline_default(8085)] pub WorldServerPort: u32,
    /// TCP port to for second world connection.
    #[serde_inline_default(8086)] pub InstanceServerPort: u16,
    /// Time after which a connection being idle on the character
    /// selection screen is disconnected.
    #[serde(default)] pub SocketTimeOutTime: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(15) }>,
    /// Time after which an idle connection is dropped while
    /// logged into the world.
    /// The client sends keepalive packets every 30 seconds. Values <= 30s are not recommended.
    #[serde(default)] pub SocketTimeOutTimeActive: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(1) }>,
    /// Server realm type.
    #[serde_inline_default(RealmType::Normal)] pub GameType: RealmType,
    #[serde_inline_default(RealmZone::Development)] pub RealmZone: RealmZone,
    #[serde_inline_default(None.into())] pub StrictPlayerNames: FlagSet<StrictName>,
    #[serde_inline_default(None.into())] pub StrictCharterNames: FlagSet<StrictName>,
    #[serde_inline_default(None.into())] pub StrictChannelNames: FlagSet<StrictName>,
    #[serde_inline_default(None.into())] pub StrictPetNames: FlagSet<StrictName>,
    #[serde(default)] pub MinPlayerName: RangedBoundedNum<u32, 1, { MAX_PLAYER_NAME as i128 }, 2>,
    #[serde(default)] pub MinCharterName: RangedBoundedNum<u32, 1, { MAX_CHARTER_NAME as i128 }, 2>,
    #[serde(default)] pub MinPetName: RangedBoundedNum<u32, 1, { MAX_PET_NAME as i128 }, 2>,
    #[serde(default)] pub Guild: pub struct WorldConfigGuild {
        #[serde(default)] pub SaveInterval: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(15) }>,
        #[serde(default)] pub ResetHour: UpperBoundedNum<u32, 23, 6>,
        #[serde(default)] pub CharterCost: LowerBoundedNum<u32, 0, 1000>,
        #[serde(default)] pub BankInitialTabs: RangedBoundedNum<u32, 0, 6, 0>,
        #[serde_inline_default([1000000, 2500000, 5000000, 10000000, 25000000, 50000000])] pub BankTabCost: [u32; 6],
        #[serde(default)] pub NewsLogRecordsCount: UpperBoundedNum<u32, { GUILD_NEWSLOG_MAX_RECORDS as i128 }, { GUILD_NEWSLOG_MAX_RECORDS as i128 }>,
        #[serde(default)] pub EventLogRecordsCount: UpperBoundedNum<u32, { GUILD_EVENTLOG_MAX_RECORDS as i128 }, { GUILD_EVENTLOG_MAX_RECORDS as i128 }>,
        #[serde(default)] pub BankEventLogRecordsCount: UpperBoundedNum<u32, { GUILD_BANKLOG_MAX_RECORDS as i128 }, { GUILD_BANKLOG_MAX_RECORDS as i128 }>,
    },
    #[serde(default)] pub ArenaTeam: pub struct WorldConfigArenaTeam {
        #[serde(default)] pub CharterCost2v2: LowerBoundedNum<u32, 0, 800000>,
        #[serde(default)] pub CharterCost3v3: LowerBoundedNum<u32, 0, 1200000>,
        #[serde(default)] pub CharterCost5v5: LowerBoundedNum<u32, 0, 2000000>,
    },
    #[serde(default)] pub CharacterCreating: pub struct WorldConfigCharacterCreating {
        #[serde(default)] pub DisabledFaction: FlagSet<CharacterCreateFactionDisabled>,
        #[serde(default)] pub DisabledRaceMask: FlagSet<CharacterCreateRaceDisabled>,
        #[serde(default)] pub DisabledClassMask: FlagSet<CharacterCreateClassDisabled>,
        // NOTE: CHARACTER_CREATING_MIN_LEVEL_FOR_HEROIC_CHARACTER in AC
        #[serde(default)] pub MinLevelForDemonHunter: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 70>,
    },
    #[serde(default)] pub CharactersPerRealm: UpperBoundedNum<u32, { MAX_CHARACTERS_PER_REALM as i128 }, { MAX_CHARACTERS_PER_REALM as i128 }>,
    #[serde_inline_default(50)] pub CharactersPerAccount: u32,
    #[serde_inline_default(49)] pub MaxWhoListReturns: u32,
    // NOTE: HEROIC_CHARACTERS_PER_REALM in AC
    #[serde(default)] pub DemonHuntersPerRealm: UpperBoundedNum<u32, { MAX_CHARACTERS_PER_REALM as i128 }, 0>,
    #[serde(default)] pub SkipCinematics: SkipCinematics,
    #[serde(default)] pub MaxPlayerLevel: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, { LEVEL_LIMIT_MAX_DEFAULT as i128 }>,
    #[serde(default)] pub StartPlayerLevel: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>,
    #[serde(default)] pub MinDualSpecLevel: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 40>,
    // NOTE: START_HEROIC_PLAYER_LEVEL in AC
    #[serde(default)] pub StartDeathKnightPlayerLevel: RangedBoundedNum<u32, 55, { LEVEL_LIMIT_MAX as i128 }, 55>,
    // NOTE: START_HEROIC_PLAYER_LEVEL in AC
    #[serde(default)] pub StartDemonHunterPlayerLevel: RangedBoundedNum<u32, 98, { LEVEL_LIMIT_MAX as i128 }, 98>,
    #[serde(default)] pub StartPlayerMoney: RangedBoundedNum<u32, 0, { MAX_MONEY_AMOUNT as i128 }, 0>,
    // NOTE: START_HEROIC_PLAYER_MONEY in AC
    #[serde(default)] pub StartDeathKnightPlayerMoney: RangedBoundedNum<u32, 0, { MAX_MONEY_AMOUNT as i128 }, 2000>,
    // NOTE: START_HEROIC_PLAYER_MONEY in AC
    #[serde(default)] pub StartDemonHunterPlayerMoney: RangedBoundedNum<u32, 0, { MAX_MONEY_AMOUNT as i128 }, 2000>,
    #[serde(default)] pub Currency: pub struct WorldConfigCurrency {
        /// run weekly on at 6am on wednesdays
        #[serde_inline_default(String::from("* 0 6 * * 3"))] pub ResetCron: String,
        #[serde_inline_default(0)] pub StartApexisCrystals: u32,
        /// must multiply by 100 of the actual amt due to precision mod
        #[serde_inline_default(20000 * 100)] pub MaxApexisCrystals: u32,
        #[serde_inline_default(0)] pub StartJusticePoints: u32,
        /// must multiply by 100 of the actual amt due to precision mod
        #[serde_inline_default(4000 * 100)] pub MaxJusticePoints: u32,
        #[serde_inline_default(55)] pub StartArtifactKnowledge: u32,
    },
    #[serde(default)] pub RecruitAFriend: pub struct WorldConfigRecruitAFriend {
        #[serde(default)] pub MaxLevel: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 85>,
        #[serde_inline_default(4)] pub MaxDifference: u32,
    },
    #[serde(default)] pub Quests: pub struct WorldConfigQuests {
        #[serde(default)] pub DailyResetTime: UpperBoundedNum<u32, 23, 3>,
        #[serde(default)] pub EnableQuestTracker: bool,
        #[serde_inline_default(4)] pub LowLevelHideDiff: u32,
        #[serde_inline_default(7)] pub HighLevelHideDiff: u32,
        #[serde(default)] pub IgnoreRaid: bool,
        #[serde(default)] pub IgnoreAutoAccept: bool,
        #[serde(default)] pub IgnoreAutoComplete: bool,
        #[serde_inline_default(true)] pub POIEnabled: bool,
    },
    #[serde(default)] pub Visibility: pub struct WorldConfigVisibility {
        #[serde(default)] pub GroupMode: GroupVisibilityMode,
        #[serde_inline_default(true)] pub ObjectSparkles: bool,
        #[serde_inline_default(true)] pub ObjectQuestMarkers: bool,
        #[serde(default)] pub DistanceContinents: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(DEFAULT_VISIBILITY_DISTANCE) }>,
        #[serde(default)] pub DistanceInstances: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(DEFAULT_VISIBILITY_INSTANCE) }>,
        #[serde(default)] pub DistanceBGArenas: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(DEFAULT_VISIBILITY_BGARENAS) }>,
        #[serde(default)] pub NotifyPeriodOnContinents: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb!(DEFAULT_VISIBILITY_NOTIFY_PERIOD) }>,
        #[serde(default)] pub NotifyPeriodInInstances: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb!(DEFAULT_VISIBILITY_NOTIFY_PERIOD) }>,
        #[serde(default)] pub NotifyPeriodInBGArenas: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb!(DEFAULT_VISIBILITY_NOTIFY_PERIOD) }>,
    },
    #[serde_inline_default(2)] pub MaxPrimaryTradeSkill: u32,
    #[serde(default)] pub MinPetitionSigns: UpperBoundedNum<u32, 4, 4>,
    #[serde_inline_default(true)] pub EnableLowLevelRegenBoost: bool,
    #[serde(default)] pub MailDeliveryDelay: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_hours!(1) }>,
    #[serde(default)] pub UpdateUptimeInterval: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(10) }>,
    #[serde_inline_default(25)] pub TeleportTimeoutNear: u32,
    #[serde_inline_default(45)] pub TeleportTimeoutFar: u32,
    #[serde(default)] pub MaxAllowedMMRDrop: LowerBoundedNum<u32, 0, 500>,
    #[serde_inline_default(true)] pub EnableLoginAfterDC: bool,
    #[serde_inline_default(true)] pub DontCacheRandomMovementPaths: bool,
    #[serde(default)] pub SkillChance: pub struct WorldConfigSkillChance {
        #[serde(default)] pub Orange: UpperBoundedNum<u32, 100, 100>,
        #[serde(default)] pub Yellow: UpperBoundedNum<u32, 100, 75>,
        #[serde(default)] pub Green: UpperBoundedNum<u32, 100, 25>,
        #[serde(default)] pub Grey: UpperBoundedNum<u32, 100, 0>,
        #[serde_inline_default(75)] pub MiningSteps: u32,
        #[serde_inline_default(75)] pub SkinningSteps: u32,
        #[serde(default)] pub Prospecting: bool,
        #[serde(default)] pub Milling: bool,
    },
    #[serde(default)] pub SkillGain: pub struct WorldConfigSkillGain {
        #[serde(default)] pub Crafting: GtZeroOrOneF32,
        #[serde(default)] pub Defense: GtZeroOrOneF32,
        #[serde(default)] pub Gathering: GtZeroOrOneF32,
        #[serde(default)] pub Weapon: GtZeroOrOneF32,
    },
    #[serde_inline_default(Some(2.into()))] pub MaxOverspeedPings: Option<LowerBoundedNum<u32, 2, 2>>,
    #[serde_inline_default(true)] pub ActivateWeather: bool,
    #[serde_inline_default(AccountTypes::SecConsole)] pub DisableWaterBreath: AccountTypes,
    #[serde(default)] pub AlwaysMaxSkillForLevel: bool,
    #[serde_inline_default(CURRENT_EXPANSION)] pub Expansion: Expansion,
    #[serde(default)] pub ChatFlood: pub struct WorldConfigChatFlood {
        #[serde_inline_default(10)] pub MessageCount: u32,
        #[serde(default)] pub MessageDelay: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(1) }>,
        #[serde_inline_default(100)] pub AddonMessageCount: u32,
        #[serde(default)] pub AddonMessageDelay: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(1) }>,
        #[serde(default)] pub MuteTime: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(10) }>,
    },
    #[serde(default)] pub ChatFakeMessagePreventing: bool,
    #[serde(default)] pub ChatMuteTimeFirstLogin: Option<LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_hours!(2) }>>,
    #[serde(default)] pub EventAnnounce: bool,
    #[serde(default)] pub CreatureFamily: pub struct WorldConfigCreatureFamily {
        #[serde(default)] pub FleeAssistanceRadius: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(30.0) }>,
        #[serde(default)] pub AssistanceRadius: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(10.0) }>,
        #[serde(default)] pub AssistanceDelay: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_ms!(1500) }>,
        #[serde(default)] pub AssistancePeriod: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_ms!(3000) }>,
        #[serde(default)] pub FleeDelay: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_ms!(7000) }>,
    },
    #[serde_inline_default(3)] pub WorldBossLevelDiff: u32,
    #[serde(default)] pub Battleground: pub struct WorldConfigBattleground {
        #[serde(default)] pub RandomResetHour: UpperBoundedNum<u32, 23, 6>,
        #[serde(default)] pub DisableQuestShareInBG: bool,
        #[serde(default)] pub DisableReadyCheckInBG: bool,
        #[serde_inline_default(true)] pub CastDeserter: bool,
        #[serde(default)] pub QueueAnnouncer: pub struct WorldConfigBattlegroundQueueAnnouncer {
            #[serde(default)] pub Enabled: bool,
            #[serde(default)] pub LimitMinLevel: RangedBoundedNum<u32, 0, { LEVEL_LIMIT_MAX as i128 }, 0>,
            #[serde_inline_default(3)] pub LimitMinPlayers: u32,
            #[serde(default)] pub SpamProtectionDelay: Option<LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(30) }>>,
            #[serde(default)] pub PlayerOnly: bool,
            #[serde(default)] pub Timed: bool,
            #[serde(default)] pub Timer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(30) }>,
            // Random Battleground Rewards
            #[serde_inline_default(27000)] pub RewardWinnerHonorFirst: u32,
            /// NOTE: BG_REWARD_WINNER_ARENA_FIRST in AC
            #[serde_inline_default(10000)] pub RewardWinnerConquestFirst: u32,
            #[serde_inline_default(13500)] pub RewardWinnerHonorLast: u32,
            /// NOTE: BG_REWARD_WINNER_ARENA_LAST in AC
            #[serde_inline_default(5000)] pub RewardWinnerConquestLast: u32,
            #[serde_inline_default(4500)] pub RewardLoserHonorFirst: u32,
            #[serde_inline_default(3500)] pub RewardLoserHonorLast: u32,
        },
        #[serde(default)] pub StoreStatisticsEnabled: bool,
        #[serde(default)] pub TrackDesertersEnabled: bool,
        #[serde(default)] pub PrematureFinishTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(5) }>,
        #[serde(default)] pub InvitationType: BattlegroundQueueInvitationType,
        #[serde(default)] pub PremadeGroupWaitForMatch: Option<LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(30) }>>,
        /// Give experience for honorable kills in battlegrounds.
        #[serde(default)] pub GiveXPForKills: bool,
        #[serde(default)] pub ReportAFKNumber: RangedBoundedNum<u32, 1, 9, 3>,
        #[serde(default)] pub ReportAFKTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(4) }>,
        #[serde(default)] pub PlayerRespawn: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(30) }>,
        #[serde(default)] pub RestorationBuffRespawn: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(20) }>,
        #[serde(default)] pub BerserkingBuffRespawn: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(120) }>,
        #[serde(default)] pub SpeedBuffRespawn: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(150) }>,
    },
    #[serde(default)] pub CalendarDeleteOldEventsHour: UpperBoundedNum<u32, 23, 6>,
    #[serde_inline_default(true)] pub DetectPosCollision: bool,
    #[serde(default)] pub Channel: pub struct WorldConfigChannel {
        #[serde_inline_default(true)] pub RestrictedLfg: bool,
        #[serde(default)] pub SilentlyGMJoin: bool,
        #[serde_inline_default(AccountTypes::SecModerator)] pub ModerationGMLevel: AccountTypes,
    },
    #[serde(default)] pub TalentsInspecting: TalentsInspectingMode,
    #[serde(default)] pub ChatStrictLinkChecking: pub struct WorldConfigChatStrictLinkChecking {
        #[serde(default)] pub Severity: ChatStrictLinkCheckingSeverity,
        #[serde(default)] pub Kick: ChatStrictLinkCheckingKick,
    },
    #[serde(default)] pub CorpseDecay: pub struct WorldConfigCorpseDecay {
        #[serde(default)] pub NORMAL: Option<LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(60) }>>,
        #[serde(default)] pub RARE: Option<LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(300) }>>,
        #[serde(default)] pub ELITE: Option<LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(300) }>>,
        #[serde(default)] pub RAREELITE: Option<LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(300) }>>,
        #[serde(default)] pub WORLDBOSS: Option<LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(3600) }>>,
    },
    #[serde(default)] pub Death: pub struct WorldConfigDeath {
        #[serde(default)] pub SicknessLevel: RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 11>,
        #[serde_inline_default(true)] pub CorpseReclaimDelayPvP: bool,
        #[serde(default)] pub CorpseReclaimDelayPvE: bool,
        #[serde_inline_default(true)] pub BonesWorld: bool,
        #[serde_inline_default(true)] pub BonesBattlegroundOrArena: bool,
    },
    #[serde_inline_default(true)] pub DieCommandMode: bool,
    #[serde(default)] pub ThreatRadius: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(60.0) }>,
    #[serde(default)] pub DeclinedNames: bool,
    #[serde(default)] pub ListenRange: pub struct WorldConfigListenRange {
        #[serde(default)] pub Say: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(25.0) }>,
        #[serde(default)] pub TextEmote: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(25.0) }>,
        #[serde(default)] pub Yell: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(300.0) }>,
    },
    #[serde(default)] pub Arena: pub struct WorldConfigArena {
        #[serde(default)] pub AutoDistributePoints: bool,
        #[serde(default)] pub QueueAnnouncer: pub struct WorldConfigArenaQueueAnnouncer {
            #[serde(default)] pub Enabled: bool,
            #[serde(default)] pub PlayerOnly: bool,
            #[serde_inline_default(ArenaQueueAnnouncerDetail::TeamName | ArenaQueueAnnouncerDetail::TeamRatings)] pub Detail: FlagSet<ArenaQueueAnnouncerDetail>,
        },
        #[serde(default)] pub PreviousOpponentsDiscardTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(2) }>,
        /// pussywizard: spoiled by implementing constant day and hour, always 7 now
        #[serde(default)] pub AutoDistributeInterval: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_days!(7) }>,
        #[serde_inline_default(10)] pub GamesRequired: u32,
        #[serde_inline_default(150)] pub MaxRatingDifference: u32,
        #[serde(default)] pub RatingDiscardTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(10) }>,
        #[serde(default)] pub RatedUpdateTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(5) }>,
        #[serde_inline_default(15)] pub ArenaSeasonID: u8,
        #[serde(default)] pub ArenaStartRating: u32,
        #[serde_inline_default(0)] pub ArenaStartPersonalRating: u32,
        #[serde_inline_default(1500)] pub ArenaStartMatchmakerRating: u32,
        #[serde_inline_default(true)] pub ArenaSeasonInProgress: bool,
        #[serde(default)] pub LogExtendedInfo: bool,
        #[serde(default)] pub WinRatingModifierLower: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(48.0) }>,
        #[serde(default)] pub WinRatingModifierUpper: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(24.0) }>,
        #[serde(default)] pub LoseRatingModifier: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(24.0) }>,
        #[serde(default)] pub MatchmakerRatingModifier: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(24.0) }>,
    },
    #[serde_inline_default(true)] pub OffhandCheckAtSpellUnlearn: bool,
    #[serde(default)] pub Creature: pub struct WorldConfigCreature {
        #[serde(default)] pub PickPocketRefillDelay: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(10) }>,
        #[serde(default)] pub MovingStopTimeForPlayer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(3) }>,
    },
    #[serde(default)] pub WaterBreathTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(3) }>,
    /// Load the CharDelete related config options
    #[serde(default)] pub CharDelete: pub struct WorldConfigCharDelete {
        #[serde(default)] pub Method: CharDeleteMethod,
        #[serde(default)] pub MinLevel: RangedBoundedNum<u32, 0, { LEVEL_LIMIT_MAX as i128 }, 0>,
        #[serde(default)] pub DeathKnightMinLevel: RangedBoundedNum<u32, 0, { LEVEL_LIMIT_MAX as i128 }, 0>,
        #[serde(default)] pub DemonHunterMinLevel: RangedBoundedNum<u32, 0, { LEVEL_LIMIT_MAX as i128 }, 0>,
        #[serde(default)] pub KeepDuration: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_days!(30) }>,
    },
    /// Load the ItemDelete related config options
    #[serde(default)] pub ItemDelete: pub struct WorldConfigItemDelete {
        #[serde(default)] pub Method: ItemDeleteMethod,
        #[serde(default)] pub Vendor: ItemDeleteMethod,
        #[serde_inline_default(ItemQuality::Rare)] pub Quality: ItemQuality,
        #[serde(default)] pub ItemLevel: Option<u32>,
    },
    /// No aggro from gray mobs
    #[serde(default)] pub NoGrayAggroAbove: Option<RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>>,
    #[serde(default)] pub NoGrayAggroBelow: Option<RangedBoundedNum<u32, 1, { LEVEL_LIMIT_MAX as i128 }, 1>>,
    #[serde(default)] pub FFAPvPTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_days!(30) }>,
    #[serde(default)] pub LootNeedBeforeGreedILvlRestriction: Option<u32>,
    #[serde(default)] pub EnablePlayerSettings: bool,
    #[serde(default)] pub JoinBGAndLFGEnabled: bool,
    #[serde_inline_default(true)] pub LeaveGroupOnLogoutEnabled: bool,
    #[serde(default)] pub ChangeFactionMaxMoney: LowerBoundedNum<f32, { f32b!(0.0) }, { f32b!(0.0) }>,
    #[serde_inline_default(true)] pub PetRankModHealth: bool,
    // pub daily_rbg_min_level_ap_reward: u32, daily_rbg_min_level_ap_reward: cfg_mgr.get("DailyRBGArenaPoints.MinLevel", || 101),
    #[serde(default)] pub AuctionHouseSearchTimeout: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(1) }>,
    #[serde(default)] pub MoveMaps: pub struct WorldConfigMoveMaps {
        /// mmap.enablePathFinding in TC
        #[serde_inline_default(true)] pub Enabled: bool,
    },
    #[serde(default)] pub vmap: pub struct WorldConfigVmap {
        #[serde_inline_default(true)] pub enableIndoorCheck: bool,
        #[serde_inline_default(true)] pub petLOS: bool,
        #[serde_inline_default(true)] pub enableLOS: bool,
        #[serde_inline_default(true)] pub enableHeight: bool,
        #[serde_inline_default(true)] pub BlizzlikePvPLOS: bool,
        #[serde_inline_default(true)] pub BlizzlikeLOSInOpenWorld: bool,
    },
    #[serde_inline_default(0)] pub HonorPointsAfterDuel: u32,
    #[serde(default)] pub ResetDuelCooldowns: bool,
    #[serde(default)] pub ResetDuelHealthMana: bool,
    #[serde(default)] pub AlwaysMaxWeaponSkill: bool,
    #[serde(default)] pub PvPToken: pub struct WorldConfigPvPToken {
        #[serde(default)] pub Enabled: bool,
        #[serde(default)] pub MapAllowType: PvPTokenMapAllowType,
        #[serde_inline_default(29434)] pub ItemID: u32,
        #[serde(default)] pub ItemCount: LowerBoundedNum<u32, 1, 1>,
    },
    #[serde(default)] pub AllowTrackBothResources: bool,
    #[serde(default)] pub NoResetTalentsCost: bool,
    #[serde_inline_default(100000)] pub ToggleXPCost: u32,
    #[serde(default)] pub ShowKickInWorld: bool,
    #[serde(default)] pub ShowMuteInWorld: bool,
    #[serde(default)] pub ShowBanInWorld: bool,
    /// Duration interval to write update time diffs to the log file.
    /// Update diff can be used as a performance indicator. Diff < 300ms: good
    /// performance. Diff > 600ms. bad performance, may be caused by high CPU usage.
    #[serde(default)] pub RecordUpdateTimeDiffInterval: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(5) }>,
    /// Only record update time diff which is greater than this value.
    #[serde(default)] pub MinRecordUpdateTimeDiff: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_ms!(100) }>,
    #[serde_inline_default(1)] MapUpdateThreads: u32,
    #[serde_inline_default(None)] CommandLookupMaxResults: Option<u32>,
    #[serde(default)] pub Warden: pub struct WorldConfigWarden {
        #[serde_inline_default(true)] pub Enabled: bool,
        #[serde_inline_default(3)] pub NumMemChecks: u32,
        #[serde_inline_default(1)] pub NumLuaChecks: u32,
        #[serde_inline_default(7)] pub NumOtherChecks: u32,
        #[serde(default)] pub BanDuration: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_days!(1) }>,
        #[serde(default)] pub ClientCheckHoldOff: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(30) }>,
        #[serde(default)] pub ClientCheckFailAction: WardenClientCheckFailAction,
        #[serde(default)] pub ClientResponseDelay: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(10) }>,
    },
    #[serde(default)] pub FeatureSystem: pub struct WorldConfigFeatureSystem {
        #[serde(default)] pub BpayStoreEnabled: bool,
        #[serde(default)] pub CharacterUndeleteCooldown: Option<LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_days!(30) }>>,
    },
    #[serde_inline_default(DungeonFinderOptions::All.into())] pub DungeonFinderOptionsMask: FlagSet<DungeonFinderOptions>,
    #[serde(default)] pub DBCEnforceItemAttributes: bool,
    #[serde(default)] pub AccountPasswordChangeSecurity: AccountPasswordChangeSecurityPolicy,
    /// Max instances per hour
    #[serde_inline_default(5)] pub AccountInstancesPerHour: u32,
    /// Announce reset of instance to whole party
    #[serde(default)] pub InstancesResetAnnounce: bool,
    #[serde(default)] pub AutoBroadcast: pub struct WorldConfigAutoBroadcast {
        #[serde(default)] pub Enabled: bool,
        #[serde(default)] pub Center: AutoBroadcastDisplayMethod,
        #[serde(default)] pub Timer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(1) }>,
        #[serde(default)] pub MinDisableLevel: RangedBoundedNum<u32, 0, { LEVEL_LIMIT_MAX as i128 }, 0>,
    },
    // /// MaxPingTime in TC/AC
    // #[serde(default)] pub DBPingInterval: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(30) }>,
    #[serde(default)] pub PlayerDump: pub struct WorldConfigPlayerDump {
        #[serde_inline_default(true)] pub DisallowPaths: bool,
        #[serde_inline_default(true)] pub DisallowOverwrite: bool,
    },
    /// Should we add quest levels to the title in the NPC dialogs?
    #[serde(default)] pub UIShowQuestLevelsInDialogs: bool,
    #[serde_inline_default(true)] pub MoveMapsEnabled: bool,
    #[serde(default)] pub Wintergrasp: pub struct WorldConfigWintergrasp {
        #[serde_inline_default(true)] pub Enabled: bool,
        #[serde_inline_default(100)] pub PlayerMax: u32,
        #[serde_inline_default(0)] pub PlayerMin: u32,
        #[serde(default)] pub PlayerMinLvl: RangedBoundedNum<u32, 0, { LEVEL_LIMIT_MAX as i128 }, 77>,
        #[serde(default)] pub BattleTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(30) }>,
        #[serde(default)] pub NoBattleTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(150) }>,
        #[serde(default)] pub CrashRestartTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(10) }>,
    },
    #[serde(default)] pub TolBarad: pub struct WorldConfigTolBarad {
        #[serde_inline_default(true)] pub Enabled: bool,
        #[serde_inline_default(100)] pub PlayerMax: u32,
        #[serde_inline_default(0)] pub PlayerMin: u32,
        #[serde(default)] pub PlayerMinLvl: RangedBoundedNum<u32, 0, { LEVEL_LIMIT_MAX as i128 }, 85>,
        #[serde(default)] pub BattleTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(15) }>,
        #[serde(default)] pub BonusTime: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(5) }>,
        #[serde(default)] pub NoBattleTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(150) }>,
        #[serde(default)] pub CrashRestartTimer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(10) }>,
    },
    #[serde(default)] pub StatsLimit: pub struct WorldConfigStatsLimit {
        #[serde(default)] pub Enabled: bool,
        #[serde(default)] pub Dodge: RangedBoundedNum<f32, { f32b!(0.0) }, { f32b!(100.0) }, { f32b!(95.0) }>,
        #[serde(default)] pub Parry: RangedBoundedNum<f32, { f32b!(0.0) }, { f32b!(100.0) }, { f32b!(95.0) }>,
        #[serde(default)] pub Block: RangedBoundedNum<f32, { f32b!(0.0) }, { f32b!(100.0) }, { f32b!(95.0) }>,
        #[serde(default)] pub Crit: RangedBoundedNum<f32, { f32b!(0.0) }, { f32b!(100.0) }, { f32b!(95.0) }>,
    },
    #[serde(default)] pub PacketSpoof: pub struct WorldConfigPacketSpoof {
        #[serde(default)] pub Policy: PacketSpoofPolicy,
        #[serde(default)] pub BanMode: PacketSpoofBanMode,
        #[serde(default)] pub BanDuration: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_days!(1) }>,
    },
    #[serde(default)] pub AllowIPBasedActionLogging: bool,
    #[serde_inline_default(true)] pub IsContinentTransportEnabled: bool,
    #[serde(default)] pub IsPreloadedContinentTransportEnabled: bool,
    /// Whether to use LoS from game objects
    #[serde_inline_default(true)] pub CheckGameObjectLoS: bool,
    #[serde(default)] pub CalculateCreatureZoneAreaData: bool,
    #[serde(default)] pub CalculateGameojectZoneAreaData: bool,
    #[serde(default)] pub BlackMarket: pub struct WorldConfigBlackMarket {
        #[serde_inline_default(true)] pub Enabled: bool,
        #[serde_inline_default(12)] pub MaxAuctions: u32,
        #[serde(default)] pub UpdatePeriod: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_days!(1) }>,
    },
    // NOTE: hirogoro@22may2024: don't support hotfix for now
    // pub hotswap_enabled: bool,
    // pub hotswap_recompiler_enabled: bool,
    // pub hotswap_early_termination_enabled: bool,
    // pub hotswap_build_file_recreation_enabled: bool,
    // pub hotswap_install_enabled: bool,
    // pub hotswap_prefix_correction_enabled: bool,

    /// prevent character rename on character customization
    #[serde(default)] pub PreventRenameCharacterOnCustomization: bool,
    /// Player can join LFG anywhere
    #[serde(default)] pub LFGLocationAll: bool,
    /// Prevent players AFK from being logged out
    #[serde(default)] pub PreventAFKLogout: bool,
    /// Allow 5-man parties to use raid warnings
    #[serde(default)] pub PartyRaidWarnings: bool,
    /// Preload all grids of all non-instanced maps
    #[serde(default)] pub PreloadAllNonInstancedMapGrids: bool,
    /// Check Invalid Position
    #[serde(default)] pub CheckInvalidPosition: pub struct WorldConfigCheckInvalidPosition {
        #[serde(default)] pub Creature: bool,
        #[serde(default)] pub GameObject: bool,
    },
    // NOTE: hirogoro@22may2024: Should we keep these? Maybe can put it to a module?
    #[serde(default)] pub ICCBuff: pub struct WorldConfigICCBuff {
        #[serde_inline_default(73822)] pub Horde: u32,
        #[serde_inline_default(73828)] pub Alliance: u32,
    },
    #[serde(default)] pub SetAllCreaturesWithWaypointMovementActive: bool,
    #[serde(default)] pub WaypointMovementStopTimeForPlayer: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(2) }>,
    #[serde(default)] pub DungeonAccessRequirements: pub struct WorldConfigDungeonAccessRequirements {
        #[serde(default)] pub PrintMode: DungeonAccessRequirementsPrintMode,
        #[serde(default)] pub PortalAvgIlevelCheck: bool,
        #[serde(default)] pub LFGLevelDBCOverride: bool,
        #[serde(default)] pub OptionalStringID: u32,
    },
    #[serde(default)] pub NpcEvadeIfTargetIsUnreachable: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(5) }>,
    #[serde(default)] pub NpcRegenHPTimeIfTargetIsUnreachable: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(10) }>,
    #[serde_inline_default(true)] pub NpcRegenHPIfTargetIsUnreachable: bool,
    #[serde(default)] pub DebugBattleground: bool,
    #[serde(default)] pub DebugArena: bool,
    #[serde_inline_default(true)] pub SetBOPItemTradeable: bool,
    /// Specifies if IP addresses can be logged to the database
    #[serde_inline_default(true)] pub AllowLoggingIPAddressesInDatabase: bool,
    /// LFG group mechanics.
    #[serde(default)] pub LFG: pub struct WorldConfigLFG {
        #[serde(default)] pub MaxKickCount: RangedBoundedNum<u32, 0, 3, 2>,
        #[serde(default)] pub KickPreventionTimer: RangedBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(15) }, { durationb_mins!(15) }>,
    },
    /// Realm Availability - CONFIG_REALM_LOGIN_ENABLED in Acore
    #[serde_inline_default(true)] pub WorldRealmAvailability: bool,
    #[serde(default)] pub Support: pub struct WorldConfigSupport {
        /// CONFIG_ALLOW_TICKETS in Acore
        #[serde_inline_default(true)] pub Enabled: bool,
        #[serde(default)] pub TicketsEnabled: bool,
        #[serde(default)] pub BugsEnabled: bool,
        #[serde(default)] pub ComplaintsEnabled: bool,
        #[serde(default)] pub SuggestionsEnabled: bool,
        #[serde(default)] pub DeletedCharacterTicketTrace: bool,
        /// CONFIG_CHANCE_OF_GM_SURVEY in Acore
        #[serde(default)] pub ChanceOfGMSurvey: RangedBoundedNum<f32, { f32b!(0.0) }, { f32b!(1.0) }, {f32b!(0.5) } >,
    },
    // pub ahbot_update_interval: u32,
}
}

macro_rules! clamp_cfg {
    ( $cfg:tt, $($input_key:ident).*, $($min_key:ident).*, $($max_key:ident).*) => {{
        let min = $cfg$( .$min_key )*;
        let max = $cfg$( .$max_key )*;
        let input = $cfg$( .$input_key )*;
        clamp_cfg!(|>, $cfg$( .$input_key )*, input, stringify!($(.$input_key)*).replace(' ', ""), min, stringify!($(.$min_key)*).replace(' ', ""), max, stringify!($(.$max_key)*).replace(' ', ""));
    }};
    (O>, $cfg:tt, $($input_key:ident).*, $($min_key:ident).*, $($max_key:ident).*) => {{
        if let Some(mut input) = $cfg$( .$input_key )* {
            let min = $cfg$( .$min_key )*;
            let max = $cfg$( .$max_key )*;
            clamp_cfg!(|>, input, input, stringify!($(.$input_key)*).replace(' ', ""), min, stringify!($(.$min_key)*).replace(' ', ""), max, stringify!($(.$max_key)*).replace(' ', ""));
        }
    }};
    (>, $cfg:tt, $($input_key:ident).*, $min:expr, $min_key:expr, $max:expr, $max_key:expr) => {{
        let input = $cfg$( .$input_key )*;
        clamp_cfg!(|>, $cfg$( .$input_key )*, input, stringify!($(.$input_key)*).replace(' ', ""), $min, $min_key, $max, $max_key);
    }};
    (|>, $input:expr, $input_value:expr, $input_key:expr, $min:expr, $min_key:expr, $max:expr, $max_key:expr) => {{
        debug_assert!($min <= $max, "min must be less than or equal to max");
        let v = if $input_value < $min {
            tracing::error!(
                target:"server.loading",
                "{input_key} ({input}) must be in range {min_key}..{max_key} ({max}..{max}), using {min}.",
                input_key=$input_key,
                input=$input_value,
                min_key=$min_key,
                min=$min,
                max=$max,
                max_key=$max_key,
            );
            $min.to_owned()
        } else if $input > $max {
            tracing::error!(
                target:"server.loading",
                "{input_key} ({input}) must be in range {min_key}..{max_key} ({min}..{max}), using {max}.",
                input_key=$input_key,
                input=$input_value,
                min_key=$min_key,
                min=$min,
                max=$max,
                max_key=$max_key,
            );
            $max.to_owned()
        } else {
            $input_value.to_owned()
        };
        use azothacore_common::bounded_nums::Assign;

        $input.assign(v.into());
    }};
}

macro_rules! clamp_cfg_max {
    ( $cfg:tt, $($input_key:ident).*, $($max_key:ident).*) => {{
        let max = $cfg$( .$max_key )*;
        let input = $cfg$( .$input_key )*;
        clamp_cfg_max!(|>, $cfg$( .$input_key )*, input, stringify!($(.$input_key)*).replace(' ', ""), max, stringify!($(.$max_key)*).replace(' ', ""));
    }};
    (|>, $input:expr, $input_value:expr, $input_key:expr, $max:expr, $max_key:expr) => {{
        debug_assert!($max == $max, "max must not be NAN");
        let v = if $input > $max {
            error!(
                target:"server.loading",
                "{input_key} ({input}) can't be more than {max_key} ({max}), using {max}.",
                input_key=$input_key,
                input=$input_value,
                max=$max,
                max_key=$max_key,
            );
            $max.to_owned()
        } else {
            $input_value.to_owned()
        };
        use azothacore_common::bounded_nums::Assign;

        $input.assign(v.into());
    }};
}

macro_rules! clamp_cfg_min {
    ( $cfg:tt, $($input_key:ident).*, $($min_key:ident).*) => {{
        let min = $cfg$( .$min_key )*;
        let input = $cfg$( .$input_key )*;

        clamp_cfg_min!(|>, $cfg$( .$input_key )*, input, stringify!($(.$input_key)*).replace(' ', ""), min, stringify!($(.$min_key)*).replace(' ', ""));
    }};
    (|>, $input:expr, $input_value:expr, $input_key:expr, $min:expr, $min_key:expr) => {{
        debug_assert!($min == $min, "min must not be NAN");
        let v = if $input_value < $min {
            error!(
                target:"server.loading",
                "{input_key} ({input}) can't be less than {min_key} ({min}), using {min}.",
                input_key=$input_key,
                input=$input_value,
                min=$min,
                min_key=$min_key,
            );
            $min.to_owned()
        } else {
            $input_value.to_owned()
        };
        use azothacore_common::bounded_nums::Assign;

        $input.assign(v.into());
    }};
}

impl Config for WorldConfig {
    fn load<P: AsRef<Path>>(config_toml: P) -> AzResult<Self> {
        let mut cfg: Self = from_env_toml(config_toml)?;
        // Load the rest of the stuff that cannot be set on deserialisation time
        if cfg.BaseMapLoadAllGrids && cfg.GridUnload {
            error!(target:"server.loading", "BaseMapLoadAllGrids enabled, but GridUnload also enabled. GridUnload must be disabled to enable base map pre-loading. Base map pre-loading disabled");
            cfg.BaseMapLoadAllGrids = false;
        }
        if cfg.InstanceMapLoadAllGrids && cfg.GridUnload {
            error!(target:"server.loading", "InstanceMapLoadAllGrids enabled, but GridUnload also enabled. GridUnload must be disabled to enable instance map pre-loading. Instance map pre-loading disabled");
            cfg.InstanceMapLoadAllGrids = false;
        }
        // must be after CONFIG_CHARACTERS_PER_REALM
        clamp_cfg_min!(cfg, CharactersPerAccount, CharactersPerRealm);
        clamp_cfg_max!(cfg, StartPlayerLevel, MaxPlayerLevel);
        clamp_cfg!(cfg, StartDeathKnightPlayerLevel, StartPlayerLevel, MaxPlayerLevel);
        clamp_cfg!(cfg, StartDemonHunterPlayerLevel, StartPlayerLevel, MaxPlayerLevel);
        clamp_cfg_max!(cfg, Currency.StartApexisCrystals, Currency.MaxApexisCrystals);
        clamp_cfg_max!(cfg, Currency.StartJusticePoints, Currency.MaxJusticePoints);
        clamp_cfg_max!(cfg, RecruitAFriend.MaxLevel, MaxPlayerLevel);
        clamp_cfg!(cfg, GM.StartLevel, StartPlayerLevel, MaxPlayerLevel);

        // always use declined names in the russian client
        cfg.DeclinedNames = cfg.RealmZone == RealmZone::Russian || cfg.DeclinedNames;
        // visibility on continents
        let max_aggro_radius = 45.0 * (*cfg.Rate.Creature.Aggro);
        clamp_cfg!(>, cfg, Visibility.DistanceContinents, max_aggro_radius, "max aggro radius", MAX_VISIBILITY_DISTANCE, "MAX_VISIBILITY_DISTANCE");
        clamp_cfg!(>, cfg, Visibility.DistanceInstances, max_aggro_radius, "max aggro radius", MAX_VISIBILITY_DISTANCE, "MAX_VISIBILITY_DISTANCE");
        clamp_cfg!(>, cfg, Visibility.DistanceBGArenas, max_aggro_radius, "max aggro radius", MAX_VISIBILITY_DISTANCE, "MAX_VISIBILITY_DISTANCE");

        clamp_cfg!(O>, cfg, NoGrayAggroAbove, StartPlayerLevel, MaxPlayerLevel);
        clamp_cfg!(O>, cfg, NoGrayAggroBelow, StartPlayerLevel, MaxPlayerLevel);

        if let (Some(above), Some(mut below)) = (cfg.NoGrayAggroAbove, cfg.NoGrayAggroBelow) {
            clamp_cfg_max!(|>, below, below, "NoGrayAggroBelow", above, "NoGrayAggroAbove");
        }
        Ok(cfg)
    }

    fn reload(&mut self, new: Self) {
        let old = mem::replace(self, new);
        if self.WorldServerPort != old.WorldServerPort {
            error!(target:"server.loading", "WorldServerPort option can't be changed on reload, using current value ({}).", old.WorldServerPort);
            self.WorldServerPort = old.WorldServerPort
        }
        if self.InstanceServerPort != old.InstanceServerPort {
            error!(target:"server.loading", "InstanceServerPort option can't be changed on reload, using current value ({}).", old.InstanceServerPort);
            self.InstanceServerPort = old.InstanceServerPort
        }
        if self.GameType != old.GameType {
            error!(target:"server.loading", "GameType option can't be changed on reload, using current value ({:?}).", old.GameType);
            self.GameType = old.GameType;
        }
        if self.RealmZone != old.RealmZone {
            error!(target:"server.loading", "RealmZone option can't be changed on reload, using current value ({:?}).", old.RealmZone);
            self.RealmZone = old.RealmZone;
        }
        if self.MaxPlayerLevel != old.MaxPlayerLevel {
            error!(target:"server.loading", "MaxPlayerLevel option can't be changed at config reload, using current value ({}).", old.MaxPlayerLevel);
            self.MaxPlayerLevel = old.MaxPlayerLevel;
        }
        if self.Expansion != old.Expansion {
            error!(target:"server.loading", "Expansion option can't be changed at config reload, using current value ({:?}).", old.Expansion);
            self.Expansion = old.Expansion;
        }
        if self.DataDir != old.DataDir {
            error!(target:"server.loading", "DataDir option can't be changed at config reload, using current value ({:?}).", old.DataDir);
            self.DataDir = old.DataDir;
        }
    }
}

impl DataDirConfig for WorldConfig {
    fn db2_dir(&self) -> PathBuf {
        self.DataDir.join("dbc")
    }

    fn cameras_dir(&self) -> PathBuf {
        self.DataDir.join("cameras")
    }

    fn gt_dir(&self) -> PathBuf {
        self.DataDir.join("gt")
    }

    fn maps_dir(&self) -> PathBuf {
        self.DataDir.join("maps")
    }

    fn mmaps_dir(&self) -> PathBuf {
        self.DataDir.join("mmaps")
    }
}

impl VmapConfig for WorldConfig {
    fn vmaps_dir(&self) -> PathBuf {
        self.DataDir.join("vmaps")
    }

    fn enable_height_calc(&self) -> bool {
        self.vmap.enableHeight
    }

    fn enable_line_of_sight_calc(&self) -> bool {
        self.vmap.enableLOS
    }
}

impl LoggingConfig for WorldConfig {
    fn retrieve_appenders(&self) -> &[LogAppender] {
        &self.Appender
    }

    fn retrieve_loggers(&self) -> &[LogLoggerConfig] {
        &self.Logger
    }

    fn retrieve_logs_dir(&self) -> PathBuf {
        self.LogsDir.clone()
    }
}

impl RealmListConfig for WorldConfig {
    fn realms_state_update_delay(&self) -> Duration {
        *self.RealmsStateUpdateDelay
    }
}
