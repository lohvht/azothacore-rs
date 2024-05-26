use flagset::flags;

use crate::game::grid::grid_defines::SIZE_OF_GRIDS;

pub const CONTACT_DISTANCE: f32 = 0.5;
pub const INTERACTION_DISTANCE: f32 = 5.5;
pub const ATTACK_DISTANCE: f32 = 5.0;
/// increase searchers
pub const VISIBILITY_COMPENSATION: f32 = 15.0;
pub const INSPECT_DISTANCE: f32 = 28.0;
/// pussywizard
pub const VISIBILITY_INC_FOR_GOBJECTS: f32 = 30.0;
/// increase searchers size in case we have large npc near cell border
pub const SPELL_SEARCHER_COMPENSATION: f32 = 30.0;
pub const TRADE_DISTANCE: f32 = 11.11;
/// max distance for visible objects
pub const MAX_VISIBILITY_DISTANCE: f32 = 250.0; // SIZE_OF_GRIDS;
pub const SIGHT_RANGE_UNIT: f32 = 50.0;
/// pussywizard: replace the use of MAX_VISIBILITY_DISTANCE in searchers, because MAX_VISIBILITY_DISTANCE is quite too big for this purpose
pub const MAX_SEARCHER_DISTANCE: f32 = 150.0;
pub const VISIBILITY_DISTANCE_INFINITE: f32 = 533.0;
pub const VISIBILITY_DISTANCE_GIGANTIC: f32 = 400.0;
pub const VISIBILITY_DISTANCE_LARGE: f32 = 200.0;
pub const VISIBILITY_DISTANCE_NORMAL: f32 = 100.0;
pub const VISIBILITY_DISTANCE_SMALL: f32 = 50.0;
pub const VISIBILITY_DISTANCE_TINY: f32 = 25.0;
/// default visible distance on continents
pub const DEFAULT_VISIBILITY_DISTANCE: f32 = 100.0;
/// default visible distance in instances
pub const DEFAULT_VISIBILITY_INSTANCE: f32 = 170.0;
/// default visible distance in BG/Arenas
pub const DEFAULT_VISIBILITY_BGARENAS: f32 = SIZE_OF_GRIDS;

/// player size, also currently used (correctly?) for any non Unit world objects
pub const DEFAULT_WORLD_OBJECT_SIZE: f64 = 0.388999998569489;
pub const DEFAULT_COMBAT_REACH: f32 = 1.5;
pub const MIN_MELEE_REACH: f32 = 2.0;
pub const NOMINAL_MELEE_RANGE: f32 = 5.0;
///center to center for players
pub const MELEE_RANGE: f32 = NOMINAL_MELEE_RANGE - MIN_MELEE_REACH * 2.0;
/// Most common value in dbc
pub const DEFAULT_COLLISION_HEIGHT: f32 = 2.03128;

pub enum TempSummonType {
    /// despawns after a specified time OR when the creature disappears
    TimedOrDeadDespawn = 1,
    /// despawns after a specified time OR when the creature dies
    TimedOrCorpseDespawn = 2,
    /// despawns after a specified time
    TimedDespawn = 3,
    /// despawns after a specified time after the creature is out of combat
    TimedDespawnOutOfCombat = 4,
    /// despawns instantly after death
    CorpseDespawn = 5,
    /// despawns after a specified time after death
    CorpseTimedDespawn = 6,
    /// despawns when the creature disappears
    DeadDespawn = 7,
    /// despawns when UnSummon() is called
    ManualDespawn = 8,
}

flags! {
    enum PhaseMasks: u32
    {
        Normal   = 0x00000001,
        Anywhere = 0xFFFFFFFF,
    }
}

flags! {
    enum NotifyFlags: u16
    {
        None              = 0x00,
        AiRelocation      = 0x01,
        VisibilityChanged = 0x02,
        All               = 0xFF,
    }
}

pub enum VisibilityDistanceType {
    Normal = 0,
    Tiny = 1,
    Small = 2,
    Large = 3,
    Gigantic = 4,
    Infinite = 5,
}
