use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use thiserror::Error;

pub mod banner;
pub mod configuration;
pub mod utils;

#[derive(Debug, FromPrimitive)]
pub enum AccountTypes {
    SecPlayer = 0,
    SecModerator = 1,
    SecGamemaster = 2,
    SecAdministrator = 3,
    /// must be always last in list, accounts must have less security level always also
    SecConsole = 4,
}

#[derive(Error, Debug, Clone)]
#[error("parse account types error: got {got}")]
pub struct AccountTypesParseError {
    got: u8,
}

impl TryFrom<u8> for AccountTypes {
    type Error = AccountTypesParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or(AccountTypesParseError { got: value })
    }
}
