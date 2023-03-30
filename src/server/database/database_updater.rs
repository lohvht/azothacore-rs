use std::{collections::BTreeSet, path::Path};

use sqlx::{MySql, MySqlConnection};
use tracing::{error, info, instrument, warn};
use walkdir::{DirEntry, WalkDir};

use crate::{
    common::configuration::{DatabaseInfo, Updates},
    server::database::{
        database_loader_utils::{apply_file, DatabaseLoaderError},
        database_update_fetcher::UpdateFetcher,
    },
};

#[instrument(skip(executor))]
pub async fn db_updater_create(executor: &mut MySqlConnection, cfg: &DatabaseInfo) -> Result<(), DatabaseLoaderError> {
    warn!("Database {} does not exist!", cfg.DatabaseName);
    info!("Creating database '{}'...", cfg.DatabaseName);
    sqlx::query(&format!(
        "CREATE DATABASE `{}` DEFAULT CHARACTER SET UTF8MB4 COLLATE utf8mb4_general_ci",
        cfg.DatabaseName
    ))
    .execute(executor)
    .await?;
    info!("Done.\n");
    Ok(())
}

#[instrument(skip(pool))]
pub async fn db_updater_populate(pool: &sqlx::Pool<MySql>, cfg: &DatabaseInfo) -> Result<(), DatabaseLoaderError> {
    let res = sqlx::query("SHOW TABLES").fetch_optional(pool).await?;
    if res.is_some() {
        return Ok(());
    }
    info!("database '{}' is empty, auto populating it...", cfg.DatabaseName);

    let dir_path = Path::new(&cfg.BaseFilePath);
    if !dir_path.is_dir() {
        error!(">> Directory '{}' not exist", dir_path.display());
        return Err(DatabaseLoaderError::NoBaseDirToPopulate {
            path: cfg.BaseFilePath.clone(),
        });
    }
    let files: Vec<DirEntry> = WalkDir::new(dir_path)
        .sort_by(|a, b| a.path().cmp(b.path()))
        .into_iter()
        .filter_map(|e| {
            let e = e.ok()?;
            if "sql" == e.path().extension()? {
                Some(e)
            } else {
                None
            }
        })
        .collect();

    if files.is_empty() {
        error!(">> In directory \"{}\" not exist '*.sql' files", dir_path.display());
        return Err(DatabaseLoaderError::NoBaseDirToPopulate {
            path: cfg.BaseFilePath.clone(),
        });
    }

    for f in files {
        apply_file(pool, f.path()).await?;
    }
    info!(">> Done!\n");
    Ok(())
}

#[instrument(skip(pool))]
pub async fn db_updater_update(
    pool: &sqlx::Pool<MySql>,
    cfg: &DatabaseInfo,
    update_cfg: &Updates,
    modules_list: &BTreeSet<String>,
) -> Result<(), DatabaseLoaderError> {
    info!("Updating {} database...", cfg.DatabaseName);

    check_update_table(pool, cfg, "updates").await?;
    check_update_table(pool, cfg, "updates_include").await?;

    let source_directory = ".".to_string();
    let uf = UpdateFetcher::new(
        source_directory,
        cfg.DBModuleName.clone(),
        modules_list.clone().into_iter(),
        update_cfg,
    );

    let (updated, recent, archived) = uf.update(pool).await?;
    let info = format!("Containing {} new and {} archived updates.", recent, archived);
    if updated > 0 {
        info!(">> {} database is up-to-date! {}", cfg.DatabaseName, info);
    } else {
        info!(">> Applied {} queries. {}", updated, info);
    }
    Ok(())
}

#[instrument(skip(pool))]
async fn check_update_table(pool: &sqlx::Pool<MySql>, cfg: &DatabaseInfo, table_name: &str) -> Result<(), DatabaseLoaderError> {
    let res = sqlx::query(&format!("SHOW TABLES LIKE '{}'", table_name))
        .bind(table_name)
        .fetch_optional(pool)
        .await?;
    if res.is_some() {
        return Ok(());
    }
    warn!("> Table '{}' not exist! Try add based table", table_name);

    let mut f = Path::new(&cfg.BaseFilePath).to_path_buf();
    f.push(format!("{table_name}.sql"));

    apply_file(pool, f).await.inspect_err(|_| {
        error!(
            "Failed apply file to database {}! Does the user (named in *.conf) have `INSERT` and `DELETE` privileges on the MySQL server?",
            &cfg.DatabaseName
        );
    })?;

    Ok(())
}
