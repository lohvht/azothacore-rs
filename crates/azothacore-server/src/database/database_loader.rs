use azothacore_common::configuration::{DatabaseInfo, DbUpdates};
use sqlx::{
    mysql::{MySqlDatabaseError, MySqlPoolOptions},
    Connection,
    MySql,
    MySqlConnection,
};
use tracing::{error, info, instrument};

use crate::database::{
    database_loader_utils::DatabaseLoaderError,
    database_updater::{db_updater_create, db_updater_populate, db_updater_update},
};

pub struct DatabaseLoader<'l, 'd, 'u> {
    updates_enabled: bool,
    modules_list:    &'l [&'l str],
    database_config: &'d DatabaseInfo,
    update_config:   &'u DbUpdates,
}

impl<'l, 'd, 'u> DatabaseLoader<'l, 'd, 'u> {
    pub fn new(updates_enabled: bool, modules_list: &'l [&'l str], database_config: &'d DatabaseInfo, update_config: &'u DbUpdates) -> Self {
        Self {
            updates_enabled,
            modules_list,
            database_config,
            update_config,
        }
    }

    #[instrument(skip(self))]
    pub async fn load(&self) -> Result<sqlx::Pool<MySql>, DatabaseLoaderError> {
        if !self.updates_enabled {
            info!("Automatic database updates are disabled for {}", self.database_config.DatabaseName);
        }
        let pool = self.open_database().await?;
        self.populate_database(&pool).await?;
        self.update_database(&pool).await?;
        Ok(pool)
    }

    #[instrument(skip(self))]
    async fn open_database(&self) -> Result<sqlx::Pool<MySql>, DatabaseLoaderError> {
        let pool = match MySqlPoolOptions::new().connect(&self.database_config.connect_url()).await {
            Ok(p) => p,
            Err(sqlx::Error::Database(e)) => {
                let e = e.try_downcast::<MySqlDatabaseError>()?;
                // i.e. ER_BAD_DB_ERRORl
                if e.number() == 1049 && self.updates_enabled && self.update_config.AutoSetup {
                    let mut conn = MySqlConnection::connect(&self.database_config.connect_url_without_db()).await?;
                    db_updater_create(&mut conn, self.database_config).await?;
                    conn.close().await?;
                }
                MySqlPoolOptions::new().connect(&self.database_config.connect_url()).await?
            },
            Err(e) => return Err(e.into()),
        };
        Ok(pool)
    }

    async fn populate_database(&self, pool: &sqlx::Pool<MySql>) -> Result<(), DatabaseLoaderError> {
        if !self.updates_enabled {
            return Ok(());
        }
        if let Err(e) = db_updater_populate(pool, self.database_config).await {
            error!("Could not populate the {} database, see log for details.", self.database_config.DatabaseName,);
            return Err(e);
        };
        Ok(())
    }

    async fn update_database(&self, pool: &sqlx::Pool<MySql>) -> Result<(), DatabaseLoaderError> {
        if !self.updates_enabled {
            return Ok(());
        }
        if let Err(e) = db_updater_update(pool, self.database_config, self.update_config, self.modules_list).await {
            error!("Could not update the {} database, see log for details.", self.database_config.DatabaseName);
            return Err(e);
        };
        Ok(())
    }
}
