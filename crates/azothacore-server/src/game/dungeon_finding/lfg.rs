use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Clone, ToPrimitive, FromPrimitive)]
pub enum LfgState {
    /// Not using LFG / LFR. LFG_STATE_NONE in TC / AC
    //
    None = 0,
    /// Rolecheck active. LFG_STATE_ROLECHECK in TC / AC
    Rolecheck = 1,
    /// Queued. LFG_STATE_QUEUED in TC / AC
    Queued = 2,
    /// Proposal active. LFG_STATE_PROPOSAL in TC / AC
    Proposal = 3,
    /// Vote kick active. LFG_STATE_BOOT in TC / AC
    Boot = 4,
    /// In LFG Group, in a Dungeon. LFG_STATE_DUNGEON in TC / AC
    Dungeon = 5,
    /// In LFG Group, in a finished Dungeon. LFG_STATE_FINISHED_DUNGEON in TC / AC
    FinishedDungeon = 6,
    /// Using Raid finder. LFG_STATE_RAIDBROWSER in TC / AC
    Raidbrowser = 7,
}
