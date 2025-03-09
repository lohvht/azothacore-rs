use azothacore_common::{az_error, AzError};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, Debug, ToPrimitive, FromPrimitive, PartialEq, PartialOrd, Ord, Eq)]
pub enum LootMethod {
    FreeForAll = 0,
    RoundRobin = 1,
    MasterLoot = 2,
    GroupLoot = 3,
    NeedBeforeGreed = 4,
    PersonalLoot = 5,
}

impl TryFrom<u8> for LootMethod {
    type Error = AzError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_u8(value).ok_or(az_error!("unable to convert number '{value}' to LootMethod"))
    }
}
