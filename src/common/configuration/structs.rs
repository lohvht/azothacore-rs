use std::{
    fs,
    io,
    ops::BitAnd,
    path::{Path, PathBuf},
};

use flagset::{flags, FlagSet};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;
use thiserror::Error;
use toml;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Error reading from filesystem: {filepath:?}, err was: {err}")]
    Filesystem {
        filepath: PathBuf,
        #[source]
        err:      io::Error,
    },
    #[error("Error decoding from TOML: {filepath:?}, err was: {err}")]
    TOMLDecode {
        filepath: PathBuf,
        #[source]
        err:      toml::de::Error,
    },
    #[error("generic error: {msg}")]
    Generic { msg: String },
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    pub worldserver: Option<WorldserverConfig>,
}

impl Config {
    pub fn toml_from_filepath<C: serde::de::DeserializeOwned, P: AsRef<Path>>(filepath: P) -> Result<C, ConfigError> {
        let contents = match fs::read_to_string(filepath.as_ref()) {
            Err(e) => {
                return Err(ConfigError::Filesystem {
                    filepath: filepath.as_ref().to_owned(),
                    err:      e,
                })
            },
            Ok(s) => s,
        };
        match toml::from_str(contents.as_str()) {
            Err(e) => Err(ConfigError::TOMLDecode {
                filepath: filepath.as_ref().to_owned(),
                err:      e,
            }),
            Ok(c) => Ok(c),
        }
    }
}

structstruck::strike! {
  #[strikethrough[serde_inline_default]]
  #[strikethrough[derive(Deserialize, DefaultFromSerde, Serialize, Clone, Debug,  PartialEq)]]
  pub struct WorldserverConfig {
    #[serde_inline_default(1)]
    pub RealmID: u32,
    #[serde_inline_default(".".to_string())]
    pub DataDir: String,
    #[serde_inline_default("".to_string())]
    pub LogsDir: String,
    #[serde_inline_default("".to_string())]
    pub TempDir: String,
    #[serde_inline_default(DatabaseInfo::default_with_info("acore_auth", "data/sql/base/db_auth", "db-auth"))]
    pub LoginDatabaseInfo: DatabaseInfo,
    #[serde_inline_default(DatabaseInfo::default_with_info("acore_world", "data/sql/base/db_world", "db-world"))]
    pub WorldDatabaseInfo: DatabaseInfo,
    #[serde_inline_default(DatabaseInfo::default_with_info("acore_characters", "data/sql/base/db_characters", "db-characters"))]
    pub CharacterDatabaseInfo: DatabaseInfo,
    #[serde_inline_default(Database::default())]
    pub Database: struct {
        #[serde_inline_default(Reconnect::default())]
        pub Reconnect: struct {
            #[serde_inline_default(15)]
            pub Seconds: u8,
            #[serde_inline_default(20)]
            pub Attempts: u8,
        }
    },
    #[serde(default)]
    pub PidFile: Option<String>,
    /// MaxPingTime In minutes
    #[serde_inline_default(30)]
    pub MaxPingTime: i32,
    #[serde_inline_default("8085".to_string())]
    pub WorldServerPort: String,
    #[serde_inline_default("0.0.0.0".to_string())]
    pub BindIP: String,
    // CMakeCommand String = ""
    // BuildDirectory String = ""
    // SourceDirectory String = ""
    // MySQLExecutable String = ""
    #[serde_inline_default(2)]
    pub ThreadPool: i32,
    #[serde_inline_default("".to_string())]
    pub IPLocationFile: String,
    #[serde_inline_default(true)]
    pub AllowLoggingIPAddressesInDatabase: bool,
    #[serde_inline_default(0)]
    pub UseProcessors: i32,
    #[serde_inline_default(true)]
    pub ProcessPriority: bool,
    #[serde_inline_default(1i32)]
    pub Compression: i32,
    #[serde_inline_default(1000)]
    pub PlayerLimit: i32,
    #[serde_inline_default(true)]
    pub SaveRespawnTimeImmediately: bool,
    #[serde_inline_default(2)]
    pub MaxOverspeedPings: i32,
    #[serde_inline_default(true)]
    pub CloseIdleConnections: bool,
    #[serde_inline_default(900000)]
    pub SocketTimeOutTime: i32,
    #[serde_inline_default(60000)]
    pub SocketTimeOutTimeActive: i32,
    #[serde_inline_default(10000)]
    pub SessionAddDelay: i32,
    #[serde_inline_default(100)]
    pub MapUpdateInterval: i32,
    #[serde_inline_default(600000)]
    pub ChangeWeatherInterval: i32,
    #[serde_inline_default(900000)]
    pub PlayerSaveInterval: i32,
    #[serde_inline_default(PlayerSave::default())]
    pub PlayerSave: struct {
    #[serde_inline_default(PlayerSaveStats::default())]
        pub Stats: struct PlayerSaveStats {
            #[serde_inline_default(0)]
            pub MinLevel: i32,
            #[serde_inline_default(true)]
            pub SaveOnlyOnLogout: bool,
        }
    },
    #[serde_inline_default(Vmap::default())]
    pub vmap: struct {
        #[serde_inline_default(true)]
        pub enableLOS: bool,
        #[serde_inline_default(true)]
        pub enableHeight: bool,
        #[serde_inline_default(true)]
        pub petLOS: bool,
        #[serde_inline_default(true)]
        pub BlizzlikePvPLOS: bool,
        #[serde_inline_default(true)]
        pub enableIndoorCheck: bool,
    },
    #[serde_inline_default(true)]
    pub DetectPosCollision: bool,
    #[serde_inline_default(true)]
    pub CheckGameObjectLoS: bool,
    #[serde_inline_default(1.5)]
    pub TargetPosRecalculateRange: f64,
    #[serde_inline_default(1i32)]
    pub UpdateUptimeInterval: i32,
    #[serde_inline_default(LogDb::default())]
    pub LogDB: struct {
        #[serde_inline_default(Opt::default())]
      pub Opt: struct {
        #[serde_inline_default(10)]
        pub ClearInterval: i32,
        #[serde_inline_default(1209600)]
        pub ClearTime: i32,
      },
    },
    #[serde_inline_default(0)]
    pub MaxCoreStuckTime: i32,
    #[serde_inline_default(true)]
    pub AddonChannel: bool,
    #[serde_inline_default(MapUpdate::default())] pub MapUpdate: struct { #[serde_inline_default(1i32)] pub Threads: i32 },
    #[serde_inline_default(false)]
    pub CleanCharacterDB: bool,
    #[serde_inline_default(0)]
    pub PersistentCharacterCleanFlags: i32,
    #[serde_inline_default(false)]
    pub PreloadAllNonInstancedMapGrids: bool,
    #[serde_inline_default(false)]
    pub SetAllCreaturesWithWaypointMovementActive: bool,
    #[serde_inline_default("".to_string())]
    pub PacketLogFile: String,
    #[serde_inline_default(0)]
    pub GameType: i32,
    #[serde_inline_default(1i32)]
    pub RealmZone: i32,
    #[serde_inline_default(World::default())] pub World: struct { #[serde_inline_default(true)] pub RealmAvailability: bool, },
    #[serde_inline_default(0)]
    pub StrictPlayerNames: i32,
    #[serde_inline_default(0)]
    pub StrictCharterNames: i32,
    #[serde_inline_default(0)]
    pub StrictPetNames: i32,
    #[serde_inline_default(Dbc::default())] pub DBC: struct{ #[serde_inline_default(255)] pub Locale: i32 },
    #[serde_inline_default(false)]
    pub DeclinedNames: bool,
    #[serde_inline_default(2)]
    pub Expansion: i32,
    #[serde_inline_default(2)]
    pub MinPlayerName: i32,
    #[serde_inline_default(2)]
    pub MinCharterName: i32,
    #[serde_inline_default(2)]
    pub MinPetName: i32,
    #[serde_inline_default(Guild::default())]
    pub Guild: struct {
        #[serde_inline_default(1000)]
        pub CharterCost: i32,
        #[serde_inline_default(100)]
        pub EventLogRecordsCount: i32,
        #[serde_inline_default(6)]
        pub ResetHour: i32,
        #[serde_inline_default(25)]
        pub BankEventLogRecordsCount: i32,
        #[serde_inline_default(false)]
        pub AllowMultipleGuildMaster: bool,
        #[serde_inline_default(0)]
        pub BankInitialTabs: i32,
        #[serde_inline_default(1000000)]
        pub BankTabCost0: i32,
        #[serde_inline_default(2500000)]
        pub BankTabCost1: i32,
        #[serde_inline_default(5000000)]
        pub BankTabCost2: i32,
        #[serde_inline_default(10000000)]
        pub BankTabCost3: i32,
        #[serde_inline_default(25000000)]
        pub BankTabCost4: i32,
        #[serde_inline_default(50000000)]
        pub BankTabCost5: i32,
    },
    #[serde_inline_default(ArenaTeam::default())]
    pub ArenaTeam: struct {
        #[serde_inline_default(CharterCost::default())]
        pub CharterCost: struct {
            #[serde(rename = "2v2")]
            #[serde_inline_default(800000)]
            pub T_2v2 :i32,
            #[serde(rename = "3v3")]
            #[serde_inline_default(1200000)]
            pub T_3v3 :i32,
            #[serde(rename = "5v5")]
            #[serde_inline_default(2000000)]
            pub T_5v5 :i32,
        },
    },
    #[serde_inline_default(49)]
    pub MaxWhoListReturns: i32,
    #[serde_inline_default(CharacterCreating::default())]
    pub CharacterCreating: struct {
        #[serde_inline_default(55)]
        pub MinLevelForHeroicCharacter: i32,
        #[serde_inline_default(Disabled::default())]
        pub Disabled: struct {
            /// Disable character creation for players based on faction
            #[serde_inline_default(0)]
            pub DisableFaction: i32,
            /// Mask of races which cannot be created by players
            #[serde_inline_default(0)]
            pub RaceMask: i32,
            /// Mask of classes which cannot be created by players
            #[serde_inline_default(0)]
            pub ClassMask: i32,
        },
    },
    #[serde_inline_default(50)]
    pub CharactersPerAccount: i32,
    #[serde_inline_default(10)]
    pub CharactersPerRealm: i32,
    #[serde_inline_default(1i32)]
    pub HeroicCharactersPerRealm: i32,
    #[serde_inline_default(0)]
    pub SkipCinematics: i32,
    #[serde_inline_default(80)]
    pub MaxPlayerLevel: i32,
    #[serde_inline_default(40)]
    pub MinDualSpecLevel: i32,
    #[serde_inline_default(1i32)]
    pub StartPlayerLevel: i32,
    #[serde_inline_default(55)]
    pub StartHeroicPlayerLevel: i32,
    #[serde_inline_default(0)]
    pub StartPlayerMoney: i32,
    #[serde_inline_default(2000)]
    pub StartHeroicPlayerMoney: i32,
    #[serde_inline_default(75000)]
    pub MaxHonorPoints: i32,
    #[serde_inline_default(0)]
    pub MaxHonorPointsMoneyPerPoint: i32,
    #[serde_inline_default(0)]
    pub StartHonorPoints: i32,
    #[serde_inline_default(10000)]
    pub MaxArenaPoints: i32,
    #[serde_inline_default(0)]
    pub StartArenaPoints: i32,
    #[serde_inline_default(RecruitAFriend::default())]
    pub RecruitAFriend: struct {
        #[serde_inline_default(60)]
        pub MaxLevel: i32,
        #[serde_inline_default(4)]
        pub MaxDifference: i32,
    },
    #[serde_inline_default(1i32)]
    pub InstantLogout: i32,
    #[serde_inline_default(0)]
    pub PreventAFKLogout: i32,
    #[serde_inline_default(4)]
    pub DisableWaterBreath: i32,
    #[serde_inline_default(false)]
    pub AllFlightPaths: bool,
    #[serde_inline_default(0)]
    pub InstantFlightPaths: i32,
    #[serde_inline_default(false)]
    pub AlwaysMaxSkillForLevel: bool,
    #[serde_inline_default(true)]
    pub ActivateWeather: bool,
    // CastUnstuck = 1
    #[serde_inline_default(Instance::default())]
    pub Instance: struct {
        #[serde_inline_default(false)]
        pub IgnoreLevel: bool,
        #[serde_inline_default(false)]
        pub IgnoreRaid: bool,
        #[serde_inline_default(false)]
        pub GMSummonPlayer: bool,
        #[serde_inline_default(4)]
        pub ResetTimeHour: i32,
        #[serde_inline_default(1800000)]
        pub UnloadDelay: i32,
        #[serde_inline_default(true)]
        pub SharedNormalHeroicId: bool,
        #[serde_inline_default(1135814400)]
        pub ResetTimeRelativeTimestamp: i32,
    },
    #[serde_inline_default(Quests::default())]
    pub Quests: struct {
        #[serde_inline_default(false)]
        pub EnableQuestTracker: bool,
        #[serde_inline_default(4)]
        pub LowLevelHideDiff: i32,
        #[serde_inline_default(7)]
        pub HighLevelHideDiff: i32,
        #[serde_inline_default(false)]
        pub IgnoreRaid: bool,
        #[serde_inline_default(false)]
        pub IgnoreAutoAccept: bool,
        #[serde_inline_default(false)]
        pub IgnoreAutoComplete: bool,
    },
    #[serde_inline_default(Calendar::default())] pub Calendar: struct{ #[serde_inline_default(6)] pub DeleteOldEventsHour: i32 },
    #[serde_inline_default(2)]
    pub MaxPrimaryTradeSkill: i32,
    #[serde_inline_default(9)]
    pub MinPetitionSigns: i32,
    #[serde_inline_default(74)]
    pub MaxGroupXPDistance: i32,
    #[serde_inline_default(100)]
    pub MaxRecruitAFriendBonusDistance: i32,
    #[serde_inline_default(3600)]
    pub MailDeliveryDelay: i32,
    #[serde_inline_default(true)]
    pub OffhandCheckAtSpellUnlearn: bool,
    #[serde_inline_default(0)]
    pub ClientCacheVersion: i32,
    #[serde_inline_default(Event::default())]
    pub Event: pub struct{ #[serde_inline_default(false)] pub Announce: bool },
    #[serde_inline_default(true)]
    pub BeepAtStart: bool,
    #[serde_inline_default(true)]
    pub FlashAtStart: bool,
    #[serde_inline_default("Welcome to an AzerothCore server.".to_string())]
    pub Motd: String,
    #[serde_inline_default(Server::default())] pub Server: pub struct{ #[serde_inline_default(0)] pub LoginInfo: i32 },
    #[serde_inline_default(Command::default())] pub Command: pub struct{ #[serde_inline_default(0)] pub LookupMaxResults: i32 },
    #[serde_inline_default(true)]
    pub AllowTickets: bool,
    #[serde_inline_default(false)]
    pub DeletedCharacterTicketTrace: bool,
    #[serde_inline_default(DungeonFinder::default())] pub DungeonFinder: pub struct{ #[serde_inline_default(5)] pub OptionsMask: i32 },
    #[serde_inline_default(5)]
    pub AccountInstancesPerHour: i32,
    #[serde_inline_default(1222964635)]
    pub BirthdayTime: i32,
    #[serde_inline_default(IsContinentTransport::default())]
    pub IsContinentTransport: pub struct{ #[serde_inline_default(true)] pub Enabled: bool },
    #[serde_inline_default(IsPreloadedContinentTransport::default())]
    pub IsPreloadedContinentTransport: pub struct{ #[serde_inline_default(false)] pub Enabled: bool },
    #[serde(default)]
    pub TOTPMasterSecret: Option<String>,
    #[serde(default)]
    pub TOTPOldMasterSecret: Option<String>,
    #[serde_inline_default(Updates::default())]
    pub Updates: struct {
        #[serde_inline_default(DatabaseTypeFlags::All.into())]
        pub EnableDatabases: FlagSet<DatabaseTypeFlags>,
        #[serde_inline_default(true)]
        pub AutoSetup: bool,
        #[serde_inline_default(true)]
        pub Redundancy: bool,
        #[serde_inline_default(false)]
        pub ArchivedRedundancy: bool,
        #[serde_inline_default(true)]
        pub AllowRehash: bool,
        #[serde_inline_default(Some(3))]
        pub CleanDeadRefMaxCount: Option<usize>,

    },
    #[serde_inline_default(Warden::default())]
    pub Warden: struct {
        #[serde_inline_default(true)]
        pub Enabled: bool,
        #[serde_inline_default(3)]
        pub NumMemChecks: i32,
        #[serde_inline_default(1i32)]
        pub NumLuaChecks: i32,
        #[serde_inline_default(7)]
        pub NumOtherChecks: i32,
        #[serde_inline_default(600)]
        pub ClientResponseDelay: i32,
        #[serde_inline_default(30)]
        pub ClientCheckHoldOff: i32,
        #[serde_inline_default(0)]
        pub ClientCheckFailAction: i32,
        #[serde_inline_default(259200)]
        pub BanDuration: i32,
    },
    #[serde_inline_default(AllowTwoSide::default())]
    pub AllowTwoSide: struct {
        #[serde_inline_default(true)]
        pub Accounts: bool,
        #[serde_inline_default(Interaction::default())]
        pub Interaction: struct {
            #[serde_inline_default(false)]
            pub Calendar: bool,
            #[serde_inline_default(false)]
            pub Chat: bool,
            #[serde_inline_default(false)]
            pub Emote: bool,
            #[serde_inline_default(false)]
            pub Channel: bool,
            #[serde_inline_default(false)]
            pub Group: bool,
            #[serde_inline_default(false)]
            pub Guild: bool,
            #[serde_inline_default(false)]
            pub Auction: bool,
            #[serde_inline_default(false)]
            pub Mail: bool,
        },
        #[serde_inline_default(false)]
        pub WhoList: bool,
        #[serde_inline_default(false)]
        pub AddFriend: bool,
        #[serde_inline_default(false)]
        pub Trade: bool,
    },
    #[serde_inline_default(true)]
    pub TalentsInspecting: bool,
    // ThreatRadius = 60
    #[serde_inline_default(30)]
    pub CreatureFamilyFleeAssistanceRadius: i32,
    #[serde_inline_default(10)]
    pub CreatureFamilyAssistanceRadius: i32,
    #[serde_inline_default(2000)]
    pub CreatureFamilyAssistanceDelay: i32,
    #[serde_inline_default(3000)]
    pub CreatureFamilyAssistancePeriod: i32,
    #[serde_inline_default(7000)]
    pub CreatureFamilyFleeDelay: i32,
    #[serde_inline_default(3)]
    pub WorldBossLevelDiff: i32,
    #[serde_inline_default(Corpse::default())]
    pub Corpse: struct {
        #[serde_inline_default(CorpseDecay::default())]
        pub Decay: struct CorpseDecay {
            #[serde_inline_default(60)]
            pub NORMAL: i32,
            #[serde_inline_default(300)]
            pub RARE: i32,
            #[serde_inline_default(300)]
            pub ELITE: i32,
            #[serde_inline_default(300)]
            pub RAREELITE: i32,
            #[serde_inline_default(3600)]
            pub WORLDBOSS: i32,
        },
    },
    #[serde_inline_default(Rate::default())]
    pub Rate: struct {
        #[serde_inline_default(RateCorpse::default())]
        pub Corpse: struct RateCorpse {
            #[serde_inline_default(Decay::default())]
            pub Decay: struct{ #[serde_inline_default(0.5)] pub Looted: f64 },
        },
        #[serde_inline_default(RateCreature::default())]
        pub Creature: struct RateCreature {
            #[serde_inline_default(1.0)]
            pub Aggro: f64,
            #[serde_inline_default(Normal::default())]
            pub Normal: struct {
                #[serde_inline_default(1.0)]
                pub Damage: f64,
                #[serde_inline_default(1.0)]
                pub SpellDamage: f64,
                #[serde_inline_default(1.0)]
                pub HP: f64,
            },
            #[serde_inline_default(RateElite::default())]
            pub Elite: struct RateElite {
                #[serde_inline_default(RateEliteElite::default())]
                pub Elite: struct RateEliteElite {
                    #[serde_inline_default(1.0)]
                    pub Damage: f64,
                    #[serde_inline_default(1.0)]
                    pub SpellDamage: f64,
                    #[serde_inline_default(1.0)]
                    pub HP: f64,
                },
                #[serde_inline_default(Rare::default())]
                pub RARE: struct {
                    #[serde_inline_default(1.0)]
                    pub Damage: f64,
                    #[serde_inline_default(1.0)]
                    pub SpellDamage: f64,
                    #[serde_inline_default(1.0)]
                    pub HP: f64,
                },
                #[serde_inline_default(Rareelite::default())]
                pub RAREELITE: struct {
                    #[serde_inline_default(1.0)]
                    pub Damage: f64,
                    #[serde_inline_default(1.0)]
                    pub SpellDamage: f64,
                    #[serde_inline_default(1.0)]
                    pub HP: f64,
                },
                #[serde_inline_default(Worldboss::default())]
                pub WORLDBOSS: struct {
                    #[serde_inline_default(1.0)]
                    pub Damage: f64,
                    #[serde_inline_default(1.0)]
                    pub SpellDamage: f64,
                    #[serde_inline_default(1.0)]
                    pub HP: f64,
                },
            },
        },
        #[serde_inline_default(1.0)]
        pub Health: f64,
        #[serde_inline_default(1.0)]
        pub Mana: f64,
        #[serde_inline_default(Rage::default())]
        pub Rage: struct {
            #[serde_inline_default(1.0)]
            pub Income: f64,
            #[serde_inline_default(1.0)]
            pub Loss: f64,
        },
        #[serde_inline_default(RunicPower::default())]
        pub RunicPower: struct {
            #[serde_inline_default(1.0)]
            pub Income: f64,
            #[serde_inline_default(1.0)]
            pub Loss: f64,
        },
        #[serde_inline_default(1.0)]
        pub Focus: f64,
        #[serde_inline_default(1.0)]
        pub Energy: f64,
        #[serde_inline_default(1.0)]
        pub Loyalty: f64,
        #[serde_inline_default(Skill::default())]
        pub Skill: struct{
            #[serde_inline_default(1.0)]
            pub Discovery: f64,
        },
        #[serde_inline_default(Drop::default())]
        pub Drop: struct {
            #[serde_inline_default(DropItem::default())]
            pub Item: struct DropItem {
                #[serde_inline_default(1.0)]
                pub Poor: f64,
                #[serde_inline_default(1.0)]
                pub Normal: f64,
                #[serde_inline_default(1.0)]
                pub Uncommon: f64,
                #[serde_inline_default(1.0)]
                pub Rare: f64,
                #[serde_inline_default(1.0)]
                pub Epic: f64,
                #[serde_inline_default(1.0)]
                pub Legendary: f64,
                #[serde_inline_default(1.0)]
                pub Artifact: f64,
                #[serde_inline_default(1.0)]
                pub Referenced: f64,
                #[serde_inline_default(1.0)]
                pub ReferencedAmount: f64,
            },
            #[serde_inline_default(1.0)]
            pub Money: f64,
        },
        #[serde_inline_default(1.0)]
        pub RewardBonusMoney: f64,
        #[serde_inline_default(SellValue::default())]
        pub SellValue: struct {
            #[serde_inline_default(SellValueItem::default())]
            pub Item: struct SellValueItem {
                #[serde_inline_default(1.0)]
                pub Poor: f64,
                #[serde_inline_default(1.0)]
                pub Normal: f64,
                #[serde_inline_default(1.0)]
                pub Uncommon: f64,
                #[serde_inline_default(1.0)]
                pub Rare: f64,
                #[serde_inline_default(1.0)]
                pub Epic: f64,
                #[serde_inline_default(1.0)]
                pub Legendary: f64,
                #[serde_inline_default(1.0)]
                pub Artifact: f64,
                #[serde_inline_default(1.0)]
                pub Heirloom: f64,
            },
        },
        #[serde_inline_default(BuyValue::default())]
        pub BuyValue: struct {
            #[serde_inline_default(BuyValueItem::default())]
            pub Item: struct BuyValueItem {
                #[serde_inline_default(1.0)]
                pub Poor: f64,
                #[serde_inline_default(1.0)]
                pub Normal: f64,
                #[serde_inline_default(1.0)]
                pub Uncommon: f64,
                #[serde_inline_default(1.0)]
                pub Rare: f64,
                #[serde_inline_default(1.0)]
                pub Epic: f64,
                #[serde_inline_default(1.0)]
                pub Legendary: f64,
                #[serde_inline_default(1.0)]
                pub Artifact: f64,
                #[serde_inline_default(1.0)]
                pub Heirloom: f64,
            },
        },
        #[serde_inline_default(Xp::default())]
        pub XP: struct {
            #[serde_inline_default(1.0)]
            pub Kill: f64,
            #[serde_inline_default(Quest::default())]
            pub Quest: struct {
                #[serde_inline_default(1.0)]
                pub General: f64,
                #[serde_inline_default(1.0)]
                pub DF: f64,
            },
            #[serde_inline_default(1.0)]
            pub Explore: f64,
            #[serde_inline_default(1.0)]
            pub Pet: f64,
            #[serde_inline_default(1.0)]
            pub BattlegroundKillAV: f64,
            #[serde_inline_default(1.0)]
            pub BattlegroundKillWSG: f64,
            #[serde_inline_default(1.0)]
            pub BattlegroundKillAB: f64,
            #[serde_inline_default(1.0)]
            pub BattlegroundKillEOTS: f64,
            #[serde_inline_default(1.0)]
            pub BattlegroundKillSOTA: f64,
            #[serde_inline_default(1.0)]
            pub BattlegroundKillIC: f64,
        },
        #[serde_inline_default(1.0)]
        pub RepairCost: f64,
        #[serde_inline_default(Rest::default())]
        pub Rest: struct {
            #[serde_inline_default(1.0)]
            pub InGame: f64,
            #[serde_inline_default(Offline::default())]
            pub Offline: struct {
                #[serde_inline_default(1.0)]
                pub InTavernOrCity: f64,
                #[serde_inline_default(1.0)]
                pub InWilderness: f64,
            },
        },
        #[serde_inline_default(Damage::default())]
        pub Damage: struct{
            #[serde_inline_default(1.0)]
            pub Fall: f64,
        },
        #[serde_inline_default(Auction::default())]
        pub Auction: struct {
            #[serde_inline_default(1.0)]
            pub Time: f64,
            #[serde_inline_default(1.0)]
            pub Deposit: f64,
            #[serde_inline_default(1.0)]
            pub Cut: f64,
        },
        #[serde_inline_default(1.0)]
        pub Honor: f64,
        #[serde_inline_default(1.0)]
        pub ArenaPoints: f64,
        #[serde_inline_default(1.0)]
        pub Talent: f64,
        #[serde_inline_default(Reputation::default())]
        pub Reputation: struct {
            #[serde_inline_default(1.0)]
            pub Gain: f64,
            #[serde_inline_default(LowLevel::default())]
            pub LowLevel: struct {
                #[serde_inline_default(1.0)]
                pub Kill: f64,
                #[serde_inline_default(1.0)]
                pub Quest: f64,
            },
            #[serde_inline_default(0.1)]
            pub RecruitAFriendBonus: f64,
        },
        #[serde_inline_default(1.0)]
        pub MoveSpeed: f64,
        #[serde_inline_default(1.0)]
        pub InstanceResetTime: f64,
        #[serde_inline_default(Pet::default())]
        pub Pet: struct{
            #[serde_inline_default(0.05)]
            LevelXP: f64,
        },
        #[serde_inline_default(MissChanceMultiplier::default())]
        pub MissChanceMultiplier: struct {
            #[serde_inline_default(11.0)]
            pub TargetCreature: f64,
            #[serde_inline_default(7.0)]
            pub TargetPlayer: f64,
            #[serde_inline_default(0.0)]
            pub OnlyAffectsPlayer: f64,
        },
    },
    #[serde_inline_default(ListenRange::default())]
    pub ListenRange: struct {
        #[serde_inline_default(40)]
        pub Say: i32,
        #[serde_inline_default(40)]
        pub TextEmote: i32,
        #[serde_inline_default(300)]
        pub Yell: i32,
    },
    #[serde_inline_default(Creature::default())]
    pub Creature: struct{
        #[serde_inline_default(180000)]
        pub MovingStopTimeForPlayer: i32,
    },
    #[serde_inline_default(120)]
    pub WaypointMovementStopTimeForPlayer: i32,
    #[serde_inline_default(5)]
    pub NpcEvadeIfTargetIsUnreachable: i32,
    #[serde_inline_default(true)]
    pub NpcRegenHPIfTargetIsUnreachable: bool,
    #[serde_inline_default(10)]
    pub NpcRegenHPTimeIfTargetIsUnreachable: i32,
    #[serde_inline_default(Creatures::default())]
    pub Creatures: struct{
        #[serde_inline_default(vec![190010, 55005, 999991, 25462, 98888, 601014, 34567, 34568])]
        pub CustomIDs: Vec<i32>,
    },
    #[serde_inline_default(true)]
    pub ChatFakeMessagePreventing: bool,
    #[serde_inline_default(ChatStrictLinkChecking::default())]
    pub ChatStrictLinkChecking: struct {
        #[serde_inline_default(0)]
        pub Severity: i32,
        #[serde_inline_default(0)]
        pub Kick: i32,
    },
    #[serde_inline_default(ChatFlood::default())]
    pub ChatFlood: struct {
        #[serde_inline_default(10)]
        pub MessageCount: i32,
        #[serde_inline_default(1i32)]
        pub MessageDelay: i32,
        #[serde_inline_default(100)]
        pub AddonMessageCount: i32,
        #[serde_inline_default(1i32)]
        pub AddonMessageDelay: i32,
        #[serde_inline_default(10)]
        pub MuteTime: i32,
    },
    #[serde_inline_default(Chat::default())]
    pub Chat: struct {
        #[serde_inline_default(false)]
        pub MuteFirstLogin: bool,
        #[serde_inline_default(120)]
        pub MuteTimeFirstLogin: i32,
    },
    #[serde_inline_default(Channel::default())]
    pub Channel: struct {
        #[serde_inline_default(true)]
        pub RestrictedLfg: bool,
        #[serde_inline_default(false)]
        pub SilentlyGMJoin: bool,
        #[serde_inline_default(1i32)]
        pub ModerationGMLevel: i32,
    },
    #[serde_inline_default(ChatLevelReq::default())]
    pub ChatLevelReq: struct {
        #[serde_inline_default(1i32)]
        pub Channel: i32,
        #[serde_inline_default(1i32)]
        pub Whisper: i32,
        #[serde_inline_default(1i32)]
        pub Say: i32,
    },
    #[serde_inline_default(1i32)]
    pub PartyLevelReq: i32,
    #[serde_inline_default(true)]
    pub AllowPlayerCommands: bool,
    #[serde_inline_default(true)]
    pub PreserveCustomChannels: bool,
    #[serde_inline_default(14)]
    pub PreserveCustomChannelDuration: i32,
    #[serde_inline_default(Gm::default())]
    pub GM: struct {
        #[serde_inline_default(2)]
        pub LoginState: i32,
        #[serde_inline_default(2)]
        pub Visible: i32,
        #[serde_inline_default(2)]
        pub Chat: i32,
        #[serde_inline_default(2)]
        pub WhisperingTo: i32,
        #[serde_inline_default(InGmList::default())]
        pub InGMList: struct{
            #[serde_inline_default(3)]
            pub Level: i32,
        },
        #[serde_inline_default(InWhoList::default())]
        pub InWhoList: struct{
            #[serde_inline_default(3)]
            pub Level: i32,
        },
        #[serde_inline_default(1i32)]
        pub StartLevel: i32,
        #[serde_inline_default(false)]
        pub AllowInvite: bool,
        #[serde_inline_default(false)]
        pub AllowFriend: bool,
        #[serde_inline_default(false)]
        pub LowerSecurity: bool,
        #[serde_inline_default(TicketSystem::default())]
        pub TicketSystem: struct{
            #[serde_inline_default(50)]
            pub ChanceOfGMSurvey: i32,
        },
    },
    #[serde_inline_default(Visibility::default())]
    pub Visibility: struct {
        #[serde_inline_default(1i32)]
        pub GroupMode: i32,
        #[serde_inline_default(Distance::default())]
        pub Distance: struct {
            #[serde_inline_default(90)]
            pub Continents: i32,
            #[serde_inline_default(120)]
            pub Instances: i32,
            #[serde_inline_default(180)]
            pub BGArenas: i32,
        },
        #[serde_inline_default(Notify::default())]
        pub Notify: struct {
            #[serde_inline_default(Period::default())]
            pub Period: struct {
                #[serde_inline_default(1000)]
                pub OnContinents: i32,
                #[serde_inline_default(1000)]
                pub InInstances: i32,
                #[serde_inline_default(1000)]
                pub InBGArenas: i32,
            },
        },
        #[serde_inline_default(true)]
        pub ObjectSparkles: bool,
        #[serde_inline_default(true)]
        pub ObjectQuestMarkers: bool,
    },
    #[serde_inline_default(WaterBreath::default())]
    pub WaterBreath: struct{
        #[serde_inline_default(180000)]
        pub Timer: u32
    },
    #[serde_inline_default(true)]
    pub EnableLowLevelRegenBoost: bool,
    #[serde_inline_default(SkillGain::default())]
    pub SkillGain: struct {
        #[serde_inline_default(1i32)]
        pub Crafting: i32,
        #[serde_inline_default(1i32)]
        pub Defense: i32,
        #[serde_inline_default(1i32)]
        pub Gathering: i32,
        #[serde_inline_default(1i32)]
        pub Weapon: i32,
    },
    #[serde_inline_default(SkillChance::default())]
    pub SkillChance: struct {
        #[serde_inline_default(false)]
        pub Prospecting: bool,
        #[serde_inline_default(false)]
        pub Milling: bool,
        #[serde_inline_default(100)]
        pub Orange: i32,
        #[serde_inline_default(75)]
        pub Yellow: i32,
        #[serde_inline_default(25)]
        pub Green: i32,
        #[serde_inline_default(0)]
        pub Grey: i32,
        #[serde_inline_default(0)]
        pub MiningSteps: i32,
        #[serde_inline_default(0)]
        pub SkinningSteps: i32,
    },
    #[serde_inline_default(DurabilityLoss::default())]
    pub DurabilityLoss: struct {
        #[serde_inline_default(false)]
        pub InPvP: bool,
        #[serde_inline_default(10)]
        pub OnDeath: i32,
    },
    #[serde_inline_default(DurabilityLossChance::default())]
    pub DurabilityLossChance: struct {
        #[serde_inline_default(0.5)]
        pub Damage: f64,
        #[serde_inline_default(0.5)]
        pub Absorb: f64,
        #[serde_inline_default(0.05)]
        pub Parry: f64,
        #[serde_inline_default(0.05)]
        pub Block: f64,
    },
    #[serde_inline_default(Death::default())]
    pub Death: struct {
        #[serde_inline_default(11)]
        pub SicknessLevel: i32,
        #[serde_inline_default(CorpseReclaimDelay::default())]
        pub CorpseReclaimDelay: struct {
            #[serde_inline_default(true)]
            pub PvP: bool,
            #[serde_inline_default(false)]
            pub PvE: bool,
        },
        #[serde_inline_default(Bones::default())]
        pub Bones: struct {
            #[serde_inline_default(true)]
            pub World: bool,
            #[serde_inline_default(true)]
            pub BattlegroundOrArena: bool,
        },
    },
    #[serde_inline_default(Die::default())]
    pub Die: struct{
        #[serde_inline_default(DieCommand::default())]
        pub Command: struct DieCommand{
            #[serde_inline_default(true)]
            pub Mode: bool,
        },
    },
    #[serde_inline_default(Stats::default())]
    pub Stats: struct {
        #[serde_inline_default(Limits::default())]
        pub Limits: struct {
            #[serde_inline_default(false)]
            pub Enable: bool,
            #[serde_inline_default(95.0)]
            pub Dodge: f64,
            #[serde_inline_default(95.0)]
            pub Parry: f64,
            #[serde_inline_default(95.0)]
            pub Block: f64,
            #[serde_inline_default(95.0)]
            pub Crit: f64,
        },
    },
    #[serde_inline_default(AutoBroadcast::default())]
    pub AutoBroadcast: struct {
      #[serde_inline_default(false)]
      pub On: bool,
      #[serde_inline_default(0)]
      pub Center: i32,
      #[serde_inline_default(60000)]
      pub Timer: i32,
      #[serde_inline_default(0)]
      pub MinDisableLevel: i32,
    },
    #[serde_inline_default(Battleground::default())]
    pub Battleground: struct {
        #[serde_inline_default(true)]
        pub CastDeserter: bool,
        #[serde_inline_default(BattlegroundQueueAnnouncer::default())]
        pub QueueAnnouncer: struct BattlegroundQueueAnnouncer {
            #[serde_inline_default(false)]
            pub Enable: bool,
            #[serde_inline_default(Limit::default())]
            pub Limit: struct {
                #[serde_inline_default(0)]
                pub MinLevel: u32,
                #[serde_inline_default(3)]
                pub MinPlayers: u32,
            },
            #[serde_inline_default(SpamProtection::default())]
            pub SpamProtection: struct{
                #[serde_inline_default(30)]
                pub Delay: u32,
            },
            #[serde_inline_default(false)]
            pub PlayerOnly: bool,
            #[serde_inline_default(false)]
            pub Timed: bool,
            #[serde_inline_default(30000)]
            pub Timer: u32,
        },
        #[serde_inline_default(300000)]
        pub PrematureFinishTimer: u32,
        #[serde_inline_default(1800000)]
        pub PremadeGroupWaitForMatch: u32,
        #[serde_inline_default(false)]
        pub GiveXPForKills: bool,
        #[serde_inline_default(Random::default())]
        pub Random: struct{
            #[serde_inline_default(6)]
            pub ResetHour: i32,
        },
        #[serde_inline_default(StoreStatistics::default())]
        pub StoreStatistics: struct{
            #[serde_inline_default(true)]
            pub Enable: bool,
        },
        #[serde_inline_default(TrackDeserters::default())]
        pub TrackDeserters: struct{
            #[serde_inline_default(true)]
            pub Enable: bool,
        },
        #[serde_inline_default(0)]
        pub InvitationType: i32,
        #[serde_inline_default(ReportAfk::default())]
        pub ReportAFK: struct {
            #[serde_inline_default(3)]
            pub Number: i32,
            #[serde_inline_default(4)]
            pub Timer: i32,
        },
        #[serde_inline_default(false)]
        pub DisableQuestShareInBG: bool,
        #[serde_inline_default(false)]
        pub DisableReadyCheckInBG: bool,
        #[serde_inline_default(30)]
        pub RewardWinnerHonorFirst: i32,
        #[serde_inline_default(25)]
        pub RewardWinnerArenaFirst: i32,
        #[serde_inline_default(15)]
        pub RewardWinnerHonorLast: i32,
        #[serde_inline_default(0)]
        pub RewardWinnerArenaLast: i32,
        #[serde_inline_default(5)]
        pub RewardLoserHonorFirst: i32,
        #[serde_inline_default(5)]
        pub RewardLoserHonorLast: i32,
        #[serde_inline_default(30)]
        pub PlayerRespawn: i32,
        #[serde_inline_default(20)]
        pub RestorationBuffRespawn: i32,
        #[serde_inline_default(120)]
        pub BerserkingBuffRespawn: i32,
        #[serde_inline_default(150)]
        pub SpeedBuffRespawn: i32,
    },
    #[serde_inline_default(Wintergrasp::default())]
    pub Wintergrasp: struct {
        #[serde_inline_default(1i32)]
        pub Enable: i32,
        #[serde_inline_default(120)]
        pub PlayerMax: i32,
        #[serde_inline_default(0)]
        pub PlayerMin: i32,
        #[serde_inline_default(77)]
        pub PlayerMinLvl: i32,
        #[serde_inline_default(30)]
        pub BattleTimer: i32,
        #[serde_inline_default(150)]
        pub NoBattleTimer: i32,
        #[serde_inline_default(10)]
        pub CrashRestartTimer: i32,
    },
    #[serde_inline_default(Arena::default())]
    pub Arena: struct {
        #[serde_inline_default(150)]
        pub MaxRatingDifference: u32,
        #[serde_inline_default(600000)]
        pub RatingDiscardTimer: u32,
        #[serde_inline_default(120000)]
        pub PreviousOpponentsDiscardTimer: u32,
        #[serde_inline_default(false)]
        pub AutoDistributePoints: bool,
        #[serde_inline_default(7)]
        pub AutoDistributeInterval: u32,
        #[serde_inline_default(10)]
        pub GamesRequired: u32,
        #[serde_inline_default(QueueAnnouncer::default())]
        pub QueueAnnouncer: struct {
            #[serde_inline_default(false)]
            pub Enable: bool,
            #[serde_inline_default(false)]
            pub PlayerOnly: bool,
        },
        #[serde_inline_default(ArenaSeason::default())]
        pub ArenaSeason: struct {
            #[serde_inline_default(8)]
            pub ID: u32,
            #[serde_inline_default(true)]
            pub InProgress: bool,
        },
        #[serde_inline_default(0)]
        pub ArenaStartRating: u32,
        #[serde_inline_default(0)]
        pub ArenaStartPersonalRating: u32,
        #[serde_inline_default(1500)]
        pub ArenaStartMatchmakerRating: u32,
        #[serde_inline_default(48.0)]
        pub ArenaWinRatingModifier1: f64,
        #[serde_inline_default(24.0)]
        pub ArenaWinRatingModifier2: f64,
        #[serde_inline_default(24.0)]
        pub ArenaLoseRatingModifier: f64,
        #[serde_inline_default(24.0)]
        pub ArenaMatchmakerRatingModifier: f64,
    },
    // Network.Threads = 1
    // Network.OutKBuff = -1
    // Network.OutUBuff = 65536
    // Network.TcpNodelay = 1
    #[serde_inline_default(Console::default())]
    pub Console: struct{
        #[serde_inline_default(true)]
        pub Enable: bool,
    },
    #[serde_inline_default(Ra::default())]
    pub Ra: struct {
        #[serde_inline_default(false)]
        pub Enable: bool,
        #[serde_inline_default("0.0.0.0".to_string())]
        pub IP: String,
        #[serde_inline_default("3443".to_string())]
        pub Port: String,
        #[serde_inline_default(3)]
        pub MinLevel: i32,
    },
    #[serde_inline_default(Soap::default())]
    pub SOAP: struct {
        #[serde_inline_default(false)]
        pub Enabled: bool,
        #[serde_inline_default("127.0.0.1".to_string())]
        pub IP: String,
        #[serde_inline_default("7878".to_string())]
        pub Port: String,
    },
    #[serde_inline_default(CharDelete::default())]
    pub CharDelete: struct {
        #[serde_inline_default(0)]
        pub Method: i32,
        #[serde_inline_default(0)]
        pub MinLevel: i32,
        #[serde_inline_default(30)]
        pub KeepDays: i32,
    },
    #[serde_inline_default(ItemDelete::default())]
    pub ItemDelete: struct {
        #[serde_inline_default(0)]
        pub Method: i32,
        #[serde_inline_default(0)]
        pub Vendor: i32,
        #[serde_inline_default(3)]
        pub Quality: i32,
        #[serde_inline_default(80)]
        pub ItemLevel: i32,
    },
    #[serde_inline_default(0)]
    pub HonorPointsAfterDuel: i32,
    #[serde_inline_default(false)]
    pub AlwaysMaxWeaponSkill: bool,
    #[serde_inline_default(PvPToken::default())]
    pub PvPToken: struct {
        #[serde_inline_default(false)]
        pub Enable: bool,
        #[serde_inline_default(4)]
        pub MapAllowType: i32,
        #[serde_inline_default(29434)]
        pub ItemID: i32,
        #[serde_inline_default(1i32)]
        pub ItemCount: i32,
    },
    #[serde_inline_default(false)]
    pub NoResetTalentsCost: bool,
    #[serde_inline_default(ToggleXp::default())]
    pub ToggleXP: struct{
        #[serde_inline_default(100000)]
        pub Cost: i32,
    },
    #[serde_inline_default(false)]
    pub ShowKickInWorld: bool,
    #[serde_inline_default(false)]
    pub ShowMuteInWorld: bool,
    #[serde_inline_default(false)]
    pub ShowBanInWorld: bool,
    #[serde_inline_default(300000)]
    pub RecordUpdateTimeDiffInterval: i32,
    #[serde_inline_default(100)]
    pub MinRecordUpdateTimeDiff: i32,
    #[serde_inline_default(PlayerStart::default())]
    pub PlayerStart: struct {
        #[serde_inline_default("".to_string())]
        pub String: String,
        #[serde_inline_default(false)]
        pub AllReputation: bool,
        #[serde_inline_default(false)]
        pub CustomSpells: bool,
        #[serde_inline_default(false)]
        pub MapsExplored: bool,
    },
    #[serde_inline_default(LevelReq::default())]
    pub LevelReq: struct {
        #[serde_inline_default(1i32)]
        pub Trade: i32,
        #[serde_inline_default(1i32)]
        pub Ticket: i32,
        #[serde_inline_default(1i32)]
        pub Auction: i32,
        #[serde_inline_default(1i32)]
        pub Mail: i32,
    },
    #[serde_inline_default(PlayerDump::default())]
    pub PlayerDump: struct {
        #[serde_inline_default(true)]
        pub DisallowPaths: bool,
        #[serde_inline_default(true)]
        pub DisallowOverwrite: bool,
    },
    #[serde_inline_default(0)]
    pub DisconnectToleranceInterval: i32,
    #[serde_inline_default(50.000000)]
    pub MonsterSight: f64,
    #[serde_inline_default(0)]
    pub StrictChannelNames: i32,
    #[serde_inline_default(25)]
    pub TeleportTimeoutNear: i32,
    #[serde_inline_default(45)]
    pub TeleportTimeoutFar: i32,
    #[serde_inline_default(500)]
    pub MaxAllowedMMRDrop: i32,
    #[serde_inline_default(true)]
    pub EnableLoginAfterDC: bool,
    #[serde_inline_default(false)]
    pub DontCacheRandomMovementPaths: bool,
    #[serde_inline_default(MoveMaps::default())]
    pub MoveMaps: struct{
        #[serde_inline_default(true)]
        pub Enable: bool,
    },
    #[serde_inline_default(Minigob::default())]
    pub Minigob: struct{
        #[serde_inline_default(Manabonk::default())]
        pub Manabonk: struct{
            #[serde_inline_default(true)]
            pub Enable: bool,
        },
    },
    #[serde_inline_default(Allow::default())]
    pub Allow: struct {
        #[serde_inline_default(Ip::default())]
        pub IP: struct {
            #[serde_inline_default(Based::default())]
            pub Based: struct{
                #[serde_inline_default(Action::default())]
                pub Action: struct{
                    #[serde_inline_default(false)]
                    pub Logging: bool,
                },
            },
        },
    },
    #[serde_inline_default(Calculate::default())]
    pub Calculate: struct {
        #[serde_inline_default(CalculateCreature::default())]
        pub Creature: struct CalculateCreature {
            #[serde_inline_default(CalculateCreatureZone::default())]
            pub Zone: struct CalculateCreatureZone{
                #[serde_inline_default(CalculateCreatureZoneArea::default())]
                pub Area: struct CalculateCreatureZoneArea{
                    #[serde_inline_default(false)]
                    pub Data: bool,
                },
            },
        },
        #[serde_inline_default(Gameoject::default())]
        pub Gameoject: struct {
            #[serde_inline_default(CalculateGameojectZone::default())]
            pub Zone: struct CalculateGameojectZone{
                #[serde_inline_default(CalculateGameojectZoneArea::default())]
                pub Area: struct CalculateGameojectZoneArea{
                    #[serde_inline_default(false)]
                    pub Data: bool,
                },
            },
        },
    },
    #[serde_inline_default(Group::default())]
    pub Group: struct {
        #[serde_inline_default(Raid::default())]
        pub Raid: struct{
            #[serde_inline_default(10)]
            pub LevelRestriction: i32,
        },
    },
    #[serde_inline_default(Lfg::default())]
    pub LFG: struct {
        #[serde_inline_default(Location::default())]
        pub Location: struct{
            #[serde_inline_default(false)]
            pub All: bool,
        },
        #[serde_inline_default(2)]
        pub MaxKickCount: i32,
        #[serde_inline_default(900)]
        pub KickPreventionTimer: i32,
    },
    #[serde_inline_default(DungeonAccessRequirements::default())]
    pub DungeonAccessRequirements: struct {
        #[serde_inline_default(1i32)]
        pub PrintMode: i32,
        #[serde_inline_default(false)]
        pub PortalAvgIlevelCheck: bool,
        #[serde_inline_default(false)]
        pub LFGLevelDBCOverride: bool,
        #[serde_inline_default(0)]
        pub OptionalStringID: i32,
    },
    #[serde_inline_default(Icc::default())]
    pub ICC: struct {
        #[serde_inline_default(Buff::default())]
        pub Buff: struct {
            #[serde_inline_default(73822)]
            pub Horde: i32,
            #[serde_inline_default(73828)]
            pub Alliance: i32,
        },
    },
    #[serde_inline_default(Item::default())]
    pub Item: struct{
        #[serde_inline_default(true)]
        pub SetItemTradeable: bool,
    },
    #[serde_inline_default(30)]
    pub FFAPvPTimer: i32,
    #[serde_inline_default(70)]
    pub LootNeedBeforeGreedILvlRestriction: i32,
    #[serde_inline_default(false)]
    pub EnablePlayerSettings: bool,
    #[serde_inline_default(JoinBgAndLfg::default())]
    pub JoinBGAndLFG: struct{
        #[serde_inline_default(false)]
        pub Enable: bool,
    },
    #[serde_inline_default(LeaveGroupOnLogout::default())]
    pub LeaveGroupOnLogout: struct{
        #[serde_inline_default(true)]
        pub Enabled: bool,
    },
    #[serde_inline_default(QuestPoi::default())]
    pub QuestPOI: struct{
        #[serde_inline_default(true)]
        pub Enabled: bool,
    },
    #[serde_inline_default(ChangeFaction::default())]
    pub ChangeFaction: struct{
        #[serde_inline_default(0)]
        pub MaxMoney: i32,
    },
    // TODO: Logging
    // Appender                           map[String]LogAppender
    // Logger                             map[String]LogLoggerConfig
    #[serde_inline_default(Log::default())]
    pub Log: struct{
        #[serde_inline_default(Async::default())]
        pub Async: struct{
            #[serde_inline_default(false)]
            pub Enable: bool,
        },
    },
    #[serde_inline_default(PacketSpoof::default())]
    pub PacketSpoof: struct {
        #[serde_inline_default(1i32)]
        pub Policy: i32,
        #[serde_inline_default(0)]
        pub BanMode: i32,
        #[serde_inline_default(86400)]
        pub BanDuration: i32,
    },
    #[serde_inline_default(Debug::default())]
    pub Debug: struct {
        #[serde_inline_default(false)]
        pub Battleground: bool,
        #[serde_inline_default(false)]
        pub Arena: bool,
    },
    #[serde_inline_default(Metric::default())]
    pub Metric: struct {
        #[serde_inline_default(false)]
        pub Enable: bool,
        #[serde_inline_default(10)]
        pub Interval: i32,
        #[serde_inline_default(ConnectionInfo::default())]
        pub ConnectionInfo: struct {
            #[serde_inline_default("127.0.0.1".to_string())]
            pub Hostname: String,
            #[serde_inline_default("8086".to_string())]
            pub Port: String,
            #[serde_inline_default("worldserver".to_string())]
            pub Database: String,
        },
        #[serde_inline_default(1i32)]
        pub OverallStatusInterval: i32,
    },
  }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum LogLevel {
    Disabled,
    Fatal,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

#[serde_inline_default]
#[derive(Deserialize, Serialize, DefaultFromSerde, Clone, Debug, PartialEq)]
#[expect(non_snake_case)]
pub struct DatabaseInfo {
    #[serde_inline_default("127.0.0.1:3306".to_string())]
    pub Address:       String,
    #[serde_inline_default("acore".to_string())]
    pub User:          String,
    #[serde_inline_default("acore".to_string())]
    pub Password:      String,
    #[serde_inline_default("".to_string())]
    pub DatabaseName:  String,
    #[serde_inline_default(1)]
    pub WorkerThreads: u32,
    #[serde_inline_default(1)]
    pub SynchThreads:  u32,
    #[serde_inline_default("".to_string())]
    pub BaseFilePath:  String,
    #[serde_inline_default("".to_string())]
    pub DBModuleName:  String,
}

impl DatabaseInfo {
    pub fn default_with_info(database: &str, base_file_path: &str, db_module_name: &str) -> Self {
        Self {
            DatabaseName: database.to_string(),
            BaseFilePath: base_file_path.to_string(),
            DBModuleName: db_module_name.to_string(),
            ..Self::default()
        }
    }

    pub fn connect_url(&self) -> String {
        format!("mysql://{}:{}@{}/{}", self.User, self.Password, self.Address, self.DatabaseName)
    }

    pub fn connect_url_without_db(&self) -> String {
        format!("mysql://{}:{}@{}", self.User, self.Password, self.Address)
    }
}

flags! {
  pub enum DatabaseTypeFlags: u8 {
    None        = 0,
    #[allow(clippy::identity_op)]
    Login       = 0b001,
    Character   = 0b010,
    World       = 0b100,
    All = (DatabaseTypeFlags::Login | DatabaseTypeFlags::Character | DatabaseTypeFlags::World).bits(),
  }
}

impl Updates {
    pub fn update_enabled(&self, update_flags: DatabaseTypeFlags) -> bool {
        self.EnableDatabases.bitand(FlagSet::from(update_flags)).bits() > 0
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use flagset::FlagSet;

    use crate::common::configuration::*;

    #[test]
    fn it_sanity_checks_database_type_flags() {
        assert_eq!(FlagSet::from(DatabaseTypeFlags::None).bits(), 0);
        assert_eq!(FlagSet::from(DatabaseTypeFlags::Login).bits(), 1);
        assert_eq!(FlagSet::from(DatabaseTypeFlags::Character).bits(), 2);
        assert_eq!(FlagSet::from(DatabaseTypeFlags::World).bits(), 4);
        assert_eq!(FlagSet::from(DatabaseTypeFlags::All).bits(), 7);
    }

    #[test]
    fn it_checks_updates_enabled() {
        let mut u = Updates {
            EnableDatabases: FlagSet::from(DatabaseTypeFlags::None),
            ..Default::default()
        };
        assert!(!u.update_enabled(DatabaseTypeFlags::Login));
        assert!(!u.update_enabled(DatabaseTypeFlags::Character));
        assert!(!u.update_enabled(DatabaseTypeFlags::World));
        assert!(!u.update_enabled(DatabaseTypeFlags::All));
        u.EnableDatabases = FlagSet::from(DatabaseTypeFlags::Login);
        assert!(u.update_enabled(DatabaseTypeFlags::Login));
        assert!(!u.update_enabled(DatabaseTypeFlags::Character));
        assert!(!u.update_enabled(DatabaseTypeFlags::World));
        assert!(u.update_enabled(DatabaseTypeFlags::All));
        u.EnableDatabases = FlagSet::from(DatabaseTypeFlags::Character);
        assert!(!u.update_enabled(DatabaseTypeFlags::Login));
        assert!(u.update_enabled(DatabaseTypeFlags::Character));
        assert!(!u.update_enabled(DatabaseTypeFlags::World));
        assert!(u.update_enabled(DatabaseTypeFlags::All));
        u.EnableDatabases = FlagSet::from(DatabaseTypeFlags::World);
        assert!(!u.update_enabled(DatabaseTypeFlags::Login));
        assert!(!u.update_enabled(DatabaseTypeFlags::Character));
        assert!(u.update_enabled(DatabaseTypeFlags::World));
        assert!(u.update_enabled(DatabaseTypeFlags::All));
        u.EnableDatabases = FlagSet::from(DatabaseTypeFlags::All);
        assert!(u.update_enabled(DatabaseTypeFlags::Login));
        assert!(u.update_enabled(DatabaseTypeFlags::Character));
        assert!(u.update_enabled(DatabaseTypeFlags::World));
        assert!(u.update_enabled(DatabaseTypeFlags::All));
    }

    #[test]
    fn it_reads_the_worldserver_toml_dist_file() {
        let dist = Config::toml_from_filepath::<Config, _>("env/dist/etc/app-worldserver.toml.dist")
            .unwrap()
            .worldserver
            .unwrap();
        let example = WorldserverConfig::default();

        fs::write("left.toml", toml::to_string(&dist).unwrap()).unwrap();
        fs::write("right.toml", toml::to_string(&example).unwrap()).unwrap();

        assert_eq!(dist, example);
    }
}
