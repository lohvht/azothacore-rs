use std::time::Duration;

use azothacore_common::configuration::{DatabaseInfo, DatabaseType, DbUpdates};
use sqlx::{mysql::MySqlDatabaseError, pool::PoolOptions};
use tracing::{error, info, instrument};

use crate::{
    database_loader_utils::DatabaseLoaderError,
    database_updater::{db_updater_create, db_updater_populate, db_updater_update},
    DbDriver,
    ExtendedDBInfo,
};

pub struct DatabaseLoader<'l, 'd, 'u> {
    modules_list:    &'l [&'l str],
    database_config: ExtendedDBInfo<'d, 'u>,
}

impl<'l, 'd, 'u> DatabaseLoader<'l, 'd, 'u> {
    pub fn new(db_type: DatabaseType, database_config: &'d DatabaseInfo, update_config: &'u DbUpdates, modules_list: &'l [&'l str]) -> Self {
        Self {
            modules_list,
            database_config: ExtendedDBInfo::new((database_config, update_config, db_type)),
        }
    }

    #[instrument(skip(self))]
    pub async fn load(&self) -> Result<sqlx::Pool<DbDriver>, DatabaseLoaderError> {
        if !self.database_config.updates_enabled() {
            info!("Automatic database updates are disabled for {}", self.database_config.DatabaseName);
        }
        let pool = self.open_database().await?;
        self.populate_database(&pool).await?;
        self.update_database(&pool).await?;
        Ok(pool)
    }

    #[instrument(skip(self))]
    async fn open_database(&self) -> Result<sqlx::Pool<DbDriver>, DatabaseLoaderError> {
        let pool = match PoolOptions::<DbDriver>::new()
            .max_connections(50)
            .idle_timeout(Some(Duration::from_secs(30)))
            .connect(&self.database_config.connect_url())
            .await
        {
            Ok(p) => p,
            Err(sqlx::Error::Database(e)) => {
                let e = e.try_downcast::<MySqlDatabaseError>()?;
                // i.e. ER_BAD_DB_ERRORl
                if e.number() == 1049 && self.database_config.updates_enabled() && self.database_config.updates.AutoSetup {
                    let conn = PoolOptions::<DbDriver>::new()
                        .max_connections(1)
                        .min_connections(1)
                        .connect(&self.database_config.connect_url_without_db())
                        .await?;
                    db_updater_create(&conn, &self.database_config).await?;
                    conn.close().await;
                }
                PoolOptions::<DbDriver>::new().connect(&self.database_config.connect_url()).await?
            },
            Err(e) => return Err(e.into()),
        };
        Ok(pool)
    }

    async fn populate_database(&self, pool: &sqlx::Pool<DbDriver>) -> Result<(), DatabaseLoaderError> {
        if !self.database_config.updates_enabled() {
            return Ok(());
        }
        if let Err(e) = db_updater_populate(pool, &self.database_config).await {
            error!("Could not populate the {} database, see log for details.", self.database_config.DatabaseName,);
            return Err(e);
        };
        Ok(())
    }

    async fn update_database(&self, pool: &sqlx::Pool<DbDriver>) -> Result<(), DatabaseLoaderError> {
        if !self.database_config.updates_enabled() {
            return Ok(());
        }
        if let Err(e) = db_updater_update(pool, &self.database_config, self.modules_list).await {
            error!("Could not update the {} database, see log for details.", self.database_config.DatabaseName);
            return Err(e);
        };
        Ok(())
    }
}