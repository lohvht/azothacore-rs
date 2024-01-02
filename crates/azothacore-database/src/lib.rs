#![feature(lint_reasons)]

pub mod database_env;
pub mod database_loader;
pub mod database_loader_utils;
pub mod database_update_fetcher;
pub mod database_updater;

use std::{ops, path::PathBuf};

use azothacore_common::configuration::{DatabaseInfo, DatabaseType, DbUpdates};
pub use hugsqlx::params;
pub use sqlx::{query, query_as, query_as_with, query_with};

/// DbDriver used in azothacore -> attempt to abstract out database specific code
/// is an alias to the underlying sqlx driver implementation used
// TODO: hirogoro@23dec2023: Can consider abstracting these out and toggle it via feature flag?
// Potentially giving users a way to specify the DB engine that they wanna use.
pub type DbDriver = sqlx::MySql;

/// Db executor used in azothacore -> attempt to abstract out database specific code
/// is an alias to the underlying sqlx executor implementation used
// TODO: hirogoro@23dec2023: Can consider abstracting these out and toggle it via feature flag?
// Potentially giving users a way to specify the DB engine that they wanna use.
pub trait DbExecutor<'c>: sqlx::MySqlExecutor<'c> {}

impl<'c, T: sqlx::MySqlExecutor<'c>> DbExecutor<'c> for T {}

#[derive(Debug)]
pub struct ExtendedDBInfo<'d, 'u> {
    info:    &'d DatabaseInfo,
    updates: &'u DbUpdates,
    db_type: DatabaseType,
}

impl<'d, 'u> ops::Deref for ExtendedDBInfo<'d, 'u> {
    type Target = DatabaseInfo;

    fn deref(&self) -> &'d Self::Target {
        self.info
    }
}

impl<'d, 'u> ExtendedDBInfo<'d, 'u> {
    fn new((info, updates, db_type): (&'d DatabaseInfo, &'u DbUpdates, DatabaseType)) -> Self {
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
