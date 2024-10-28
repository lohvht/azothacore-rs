pub mod database_env;
pub mod database_loader;
pub mod database_loader_utils;

use std::{ops, path::PathBuf};

use azothacore_common::configuration::{DatabaseInfo, DatabaseType, DbUpdates};
use sqlx::{pool::PoolConnection, MySqlConnection};
pub use sqlx::{query, query_as, query_as_with, query_with};

/// DbDriver used in azothacore -> attempt to abstract out database specific code
/// is an alias to the underlying sqlx driver implementation used
// TODO: hirogoro@23dec2023: Can consider abstracting these out and toggle it via feature flag?
// Potentially giving users a way to specify the DB engine that they wanna use.
pub type DbDriver = sqlx::MySql;

pub type DbPoolConnection = PoolConnection<DbDriver>;

pub type DbConnection = MySqlConnection;
/// Db executor used in azothacore -> attempt to abstract out database specific code
/// is an alias to the underlying sqlx executor implementation used
// TODO: hirogoro@23dec2023: Can consider abstracting these out and toggle it via feature flag?
// Potentially giving users a way to specify the DB engine that they wanna use.
pub trait DbExecutor<'c>: sqlx::MySqlExecutor<'c> {}

impl<'c, T: sqlx::MySqlExecutor<'c>> DbExecutor<'c> for T {}

pub trait DbAcquire<'c>: sqlx::Acquire<'c, Database = DbDriver> {}

impl<'c, T: sqlx::Acquire<'c, Database = DbDriver>> DbAcquire<'c> for T {}

#[derive(Debug)]
pub struct ExtendedDBInfo {
    info:    DatabaseInfo,
    updates: DbUpdates,
    db_type: DatabaseType,
}

impl ops::Deref for ExtendedDBInfo {
    type Target = DatabaseInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl ExtendedDBInfo {
    fn new(info: DatabaseInfo, updates: DbUpdates, db_type: DatabaseType) -> Self {
        Self { info, updates, db_type }
    }

    fn updates_enabled(&self) -> bool {
        self.updates.EnableDatabases.contains(self.db_type)
    }

    fn base_files_dir(&self) -> PathBuf {
        self.db_type
            .base_files_directory()
            .expect("get base files by db type failed, expected to be one of the given DB and not All")
    }

    fn db_module_name(&self) -> &str {
        self.db_type
            .db_module_name()
            .expect("get db_module_name by db type failed, db type is expected to be one of the given DB and not All")
    }
}

cfg_if::cfg_if! {
if #[cfg(feature = "postgres")] {
    pub type DbArguments = sqlx::postgres::PgArguments;
} else if #[cfg(feature = "mysql")] {
    pub type DbArguments = sqlx::mysql::MySqlArguments;
} else {
    pub type DbArguments = sqlx::sqlite::SqliteArguments;
}
}

#[macro_export]
macro_rules! args {
    ($($arg:expr),*) => {
        {
            #[allow(unused_imports)]
            use sqlx::Arguments;
            use $crate::DbArguments;

            #[allow(unused_labels)]
            'ret: {
                #[allow(unused_mut)]
                let mut args = DbArguments::default();
                $(
                    let res = args.add($arg);
                    if let Err(e) = res {
                        break 'ret Err(sqlx::Error::Encode(e));
                    }
                )*
                Ok::<_, sqlx::Error>(args)
            }
        }
    };
}

#[macro_export]
macro_rules! args_unwrap {
    ($($arg:expr),*) => {
        {
            $crate::args!($($arg),*)
        }.unwrap()
    };
}
