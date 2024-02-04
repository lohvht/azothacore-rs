use tracing::error;

pub mod account_mgr;
pub mod battlenet_account_mgr;
pub mod rbac;

#[derive(Debug, thiserror::Error, PartialEq)]
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
    DbInternalError(String),
    #[error("AOR_ACCOUNT_BAD_LINK")]
    AccountBadLink,
}

impl From<sqlx::Error> for AccountOpError {
    fn from(value: sqlx::Error) -> Self {
        error!(target: "sql::sql", cause=%value, "DB error on account related operation");
        Self::DbInternalError(value.to_string())
    }
}

pub type AccountOpResult<T> = Result<T, AccountOpError>;
