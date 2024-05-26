#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum BattlegroundQueueInvitationType {
    #[default]
    NoBalance = 0, // no balance: N+M vs N players
    Balanced = 1, // teams balanced: N+1 vs N players
    Even = 2,     // teams even: N vs N players
}
