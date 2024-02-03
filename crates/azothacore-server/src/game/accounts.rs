use tracing::error;

use self::rbac::RbacCommandError;

pub mod account_mgr;
pub mod battlenet_account_mgr;
pub mod rbac;

#[derive(Debug, thiserror::Error)]
pub enum AccountOpError {
    #[error("AOR_NAME_TOO_LONG")]
    NameTooLong,
    #[error("AOR_PASS_TOO_LONG")]
    PassTooLong,
    #[error("AOR_EMAIL_TOO_LONG")]
    EmailTooLong,
    #[error("AOR_NAME_ALREADY_EXIST")]
    NameAlreadyExist,
    #[error("AOR_NAME_NOT_EXIST")]
    NameNotExist,
    #[error("AOR_DB_INTERNAL_ERROR: {0}")]
    DbInternalError(#[from] sqlx::Error),
    #[error("AOR_ACCOUNT_BAD_LINK")]
    AccountBadLink,
}

pub type AccountOpResult<T> = Result<T, AccountOpError>;

#[derive(sqlx::FromRow)]
struct DbEmail {
    email: String,
}

#[derive(sqlx::FromRow)]
struct DbId {
    id: u32,
}

#[derive(sqlx::FromRow)]
struct DbBattlenetAccount {
    battlenet_account: u32,
}

#[derive(sqlx::FromRow)]
struct DbBnetMaxIndex {
    bnet_max_index: u8,
}
