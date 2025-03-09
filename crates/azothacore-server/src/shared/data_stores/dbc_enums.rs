#[allow(non_camel_case_types)]
mod wow7_3_5_26972;

use flagset::{flags, FlagSet};
use num::{FromPrimitive, Num, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};
use thiserror::Error;
pub use wow7_3_5_26972::*;

#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, Debug, ToPrimitive, FromPrimitive, PartialEq, PartialOrd, Ord, Eq)]
pub enum CharBaseSectionVariation {
    Skin = 0,
    Face = 1,
    FacialHair = 2,
    Hair = 3,
    Underwear = 4,
    CustomDisplay1 = 5,
    CustomDisplay2 = 6,
    CustomDisplay3 = 7,
}

#[derive(Error, Debug, Clone)]
#[error("CharBaseSectionVariationError: got {got}")]
pub struct CharBaseSectionVariationError {
    got: u8,
}

impl TryFrom<u8> for CharBaseSectionVariation {
    type Error = CharBaseSectionVariationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or(CharBaseSectionVariationError { got: value })
    }
}

pub const BATTLE_PET_SPECIES_MAX_ID: usize = 2164;

#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, Debug, ToPrimitive, FromPrimitive, PartialEq, PartialOrd, Ord, Eq)]
pub enum CharSectionType {
    SkinLowRes = 0,
    FaceLowRes = 1,
    FacialHairLowRes = 2,
    HairLowRes = 3,
    UnderwearLowRes = 4,
    Skin = 5,
    Face = 6,
    FacialHair = 7,
    Hair = 8,
    Underwear = 9,
    CustomDisplay1LowRes = 10,
    CustomDisplay1 = 11,
    CustomDisplay2LowRes = 12,
    CustomDisplay2 = 13,
    CustomDisplay3LowRes = 14,
    CustomDisplay3 = 15,
}

#[derive(Error, Debug, Clone)]
#[error("CharSectionTypeError: got {got}")]
pub struct CharSectionTypeError {
    got: u8,
}

impl TryFrom<u8> for CharSectionType {
    type Error = CharSectionTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or(CharSectionTypeError { got: value })
    }
}

/// Powers in TC/AC
#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, Debug, ToPrimitive, FromPrimitive, PartialEq, PartialOrd, Ord, Eq)]
pub enum Power {
    Mana = 0,
    Rage = 1,
    Focus = 2,
    Energy = 3,
    ComboPoints = 4,
    Runes = 5,
    RunicPower = 6,
    SoulShards = 7,
    LunarPower = 8,
    HolyPower = 9,
    AlternatePower = 10, // Used in some quests
    Maelstrom = 11,
    Chi = 12,
    Insanity = 13,
    BurningEmbers = 14,
    DemonicFury = 15,
    ArcaneCharges = 16,
    Fury = 17,
    Pain = 18,
    All = 127,   // default for class?
    Health = -2, // (-2 as signed value)
}

#[derive(Error, Debug, Clone)]
#[error("PowersError: got {got}")]
pub struct PowersError {
    got: i8,
}

impl TryFrom<i8> for Power {
    type Error = PowersError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        FromPrimitive::from_i8(value).ok_or(PowersError { got: value })
    }
}

flags! {
    /// Class value is index in ChrClasses.dbc
    ///
    /// Classes in TC/AC
    #[derive(serde::Deserialize, serde::Serialize, PartialOrd, Ord)]
    pub enum Class: u32 {
        None        = 1 << 0,
        Warrior     = 1 << 1,
        Paladin     = 1 << 2,
        Hunter      = 1 << 3,
        Rogue       = 1 << 4,
        Priest      = 1 << 5,
        DeathKnight = 1 << 6,
        Shaman      = 1 << 7,
        Mage        = 1 << 8,
        Warlock     = 1 << 9,
        Monk        = 1 << 10,
        Druid       = 1 << 11,
        DemonHunter = 1 << 12,
    }
}

impl Class {
    pub fn to_num<N: Num>(&self) -> N {
        let mut bits = FlagSet::from(*self).bits();
        let mut res = N::zero();
        while bits > 1 {
            res = res + N::one();
            bits >>= 1;
        }
        res
    }
}

#[derive(Error, Debug, Clone)]
#[error("ClassError: got {got}")]
pub struct ClassError {
    pub got: u32,
}

impl TryFrom<u8> for Class {
    type Error = ClassError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let Ok(fs) = FlagSet::new(1 << value) else {
            return Err(ClassError { got: value.into() });
        };
        let mut v = None;
        for f in fs.into_iter() {
            if v.is_some() {
                return Err(ClassError { got: value.into() });
            }
            v = Some(f);
        }
        let Some(v) = v else { return Err(ClassError { got: value.into() }) };
        Ok(v)
    }
}

flags! {
    pub enum ChrSpecializationFlag: u32 {
        Caster              = 0x01,
        Ranged              = 0x02,
        Melee               = 0x04,
        Unknown             = 0x08,
        DualWieldTwoHanded  = 0x10,     // used for CUnitDisplay::SetSheatheInvertedForDualWield
        PetOverrideSpec     = 0x20,
        Recommended         = 0x40,
    }
}

#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, Debug, ToPrimitive, FromPrimitive, PartialEq, PartialOrd, Ord, Eq)]
pub enum Gender {
    Unknown = -1,
    Male = 0,
    Female = 1,
    None = 2,
}

#[derive(Error, Debug, Clone)]
#[error("GenderError: got {got}")]
pub struct GenderError {
    got: u8,
}

impl TryFrom<u8> for Gender {
    type Error = GenderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or(GenderError { got: value })
    }
}

/// ChrRaces.dbc (6.0.2.18988)
#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, Debug, ToPrimitive, FromPrimitive, PartialEq, PartialOrd, Ord, Eq)]
pub enum Race {
    None = 0,
    Human = 1,
    Orc = 2,
    Dwarf = 3,
    Nightelf = 4,
    UndeadPlayer = 5,
    Tauren = 6,
    Gnome = 7,
    Troll = 8,
    Goblin = 9,
    Bloodelf = 10,
    Draenei = 11,
    FelOrc = 12,
    Naga = 13,
    Broken = 14,
    Skeleton = 15,
    Vrykul = 16,
    Tuskarr = 17,
    ForestTroll = 18,
    Taunka = 19,
    NorthrendSkeleton = 20,
    IceTroll = 21,
    Worgen = 22,
    Gilnean = 23,
    PandarenNeutral = 24,
    PandarenAlliance = 25,
    PandarenHorde = 26,
    Nightborne = 27,
    HighmountainTauren = 28,
    VoidElf = 29,
    LightforgedDraenei = 30,
}

#[derive(Error, Debug, Clone)]
#[error("RaceError: got {got}")]
pub struct RaceError {
    got: u8,
}

impl TryFrom<u8> for Race {
    type Error = RaceError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or(RaceError { got: value })
    }
}

/// ItemClass in TC/AC
#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, Debug, ToPrimitive, FromPrimitive, PartialEq, PartialOrd, Ord, Eq)]
pub enum ItemClassID {
    Consumable = 0,
    Container = 1,
    Weapon = 2,
    Gem = 3,
    Armor = 4,
    Reagent = 5,
    Projectile = 6,
    TradeGoods = 7,
    ItemEnhancement = 8,
    Recipe = 9,
    Money = 10, // OBSOLETE
    Quiver = 11,
    Quest = 12,
    Key = 13,
    Permanent = 14, // OBSOLETE
    Miscellaneous = 15,
    Glyph = 16,
    BattlePets = 17,
    WowToken = 18,
}

#[derive(Error, Debug, Clone)]
#[error("ItemClassIDError: got {got}")]
pub struct ItemClassIDError {
    got: u8,
}

impl TryFrom<u8> for ItemClassID {
    type Error = ItemClassIDError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or(ItemClassIDError { got: value })
    }
}

#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, Debug, ToPrimitive, FromPrimitive, PartialEq, PartialOrd, Ord, Eq)]
pub enum QuestPackageFilter {
    /// Players can select this quest reward if it matches their selected loot specialization
    LootSpecialization = 0,
    /// Players can select this quest reward if it matches their class
    Class = 1,
    /// Players can select this quest reward if no class/loot_spec rewards are available
    Unmatched = 2,
    /// Players can always select this quest reward
    Everyone = 3,
}

#[derive(Error, Debug, Clone)]
#[error("QuestPackageFilterError: got {got}")]
pub struct QuestPackageFilterError {
    got: u8,
}

impl TryFrom<u8> for QuestPackageFilter {
    type Error = QuestPackageFilterError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or(QuestPackageFilterError { got: value })
    }
}

flags! {
    pub enum TaxiNodeFlags: u8 {
        Alliance           = 0x01,
        Horde              = 0x02,
        UseFavoriteMount   = 0x10,
    }
}

flags! {
    pub enum TaxiPathNodeFlags: u8 {
        Teleport    = 0x1,
        Stop        = 0x2
    }
}

flags! {
    pub enum WorldMapTransformsFlags: u8 {
        Dungeon   = 0x04
    }
}

/// Lua_IsInInstance
/// MapTypes in TC / AC
#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, Debug, ToPrimitive, FromPrimitive, PartialEq, PartialOrd, Ord, Eq)]
pub enum MapType {
    /// none
    Common = 0,
    /// party
    Instance = 1,
    /// raid
    Raid = 2,
    /// pvp
    Battleground = 3,
    /// arena
    Arena = 4,
    /// scenario
    Scenario = 5,
}

impl MapType {
    pub fn to_num(&self) -> u8 {
        self.to_u8().unwrap_or_else(|| panic!("MapType should never fail to become primitive {self:?}"))
    }
}

/// enum Difficulty in TC / AC
#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, Debug, ToPrimitive, FromPrimitive, PartialEq, PartialOrd, Ord, Eq)]
pub enum DifficultyID {
    None = 0,
    Normal = 1,
    Heroic = 2,
    _10N = 3,
    _25N = 4,
    _10HC = 5,
    _25HC = 6,
    LFR = 7,
    MythicKeystone = 8,
    _40 = 9,
    _3ManScenarioHc = 11,
    _3ManScenarioN = 12,
    NormalRaid = 14,
    HeroicRaid = 15,
    MythicRaid = 16,
    LFRNew = 17,
    EventRaid = 18,
    EventDungeon = 19,
    EventScenario = 20,
    Mythic = 23,
    Timewalking = 24,
    WorldPVPScenario = 25,
    _5ManScenarioN = 26,
    _20ManScenarioN = 27,
    PVEVPScenario = 29,
    EventScenario6 = 30,
    WorldPvpScenario2 = 32,
    TimewalkingRaid = 33,
    PVP = 34,
}

flags! {
    pub enum DifficultyFlag: u8 {
        Heroic = 0x01,
        Default = 0x02,
        /// Player can select this difficulty in dropdown menu
        CanSelect = 0x04,
        ChallengeMode = 0x08,
        Legacy = 0x20,
        /// Controls icon displayed on minimap when inside the instance
        DisplayHeroic = 0x40,
        /// Controls icon displayed on minimap when inside the instance
        DisplayMythic = 0x80,
    }
}
