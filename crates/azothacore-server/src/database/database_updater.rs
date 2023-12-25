use sqlx::Connection;
use tracing::{error, info, instrument, warn};
use walkdir::{DirEntry, WalkDir};

use crate::database::{
    database_loader_utils::{apply_file, DatabaseLoaderError},
    database_update_fetcher::UpdateFetcher,
    query,
    DbDriver,
    DbExecutor,
    ExtendedDBInfo,
};

#[instrument(skip(executor))]
pub async fn db_updater_create<'e, E: DbExecutor<'e>>(executor: E, cfg: &ExtendedDBInfo<'_, '_>) -> Result<(), DatabaseLoaderError> {
    warn!("Database {} does not exist!", cfg.DatabaseName);
    info!("Creating database '{}'...", cfg.DatabaseName);
    query(&format!(
        "CREATE DATABASE `{}` DEFAULT CHARACTER SET UTF8MB4 COLLATE utf8mb4_general_ci",
        cfg.DatabaseName
    ))
    .execute(executor)
    .await?;
    info!("Done.\n");
    Ok(())
}

#[instrument(skip(conn))]
pub async fn db_updater_populate<'a, A: sqlx::Acquire<'a, Database = DbDriver>>(conn: A, cfg: &ExtendedDBInfo<'_, '_>) -> Result<(), DatabaseLoaderError> {
    let mut conn = conn.acquire().await?;

    let res = query("SHOW TABLES").fetch_optional(&mut *conn).await?;
    if res.is_some() {
        return Ok(());
    }
    info!("database '{}' is empty, auto populating it...", cfg.DatabaseName);

    let dir_path = cfg.base_files_dir();
    if !dir_path.is_dir() {
        let path = format!("{}", dir_path.display());
        error!(">> Directory '{path}' not exist");
        return Err(DatabaseLoaderError::NoBaseDirToPopulate { path });
    }
    let files: Vec<DirEntry> = WalkDir::new(&dir_path)
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
        let path = format!("{}", dir_path.display());
        error!(">> In directory \"{path}\" not exist '*.sql' files");
        return Err(DatabaseLoaderError::NoBaseDirToPopulate { path });
    }

    for f in files {
        conn.transaction(|tx| {
            Box::pin(async move {
                apply_file(&mut **tx, f.path()).await?;
                Ok::<_, DatabaseLoaderError>(())
            })
        })
        .await?;
    }
    info!(">> Done!\n");
    Ok(())
}

#[instrument(skip(conn))]
pub async fn db_updater_update<'a, A: sqlx::Acquire<'a, Database = DbDriver>>(
    conn: A,
    cfg: &ExtendedDBInfo<'_, '_>,
    modules_list: &[&str],
) -> Result<(), DatabaseLoaderError> {
    let mut conn = conn.acquire().await?;

    info!("Updating {} database...", cfg.DatabaseName);

    check_update_table(&mut *conn, cfg, "updates").await?;
    check_update_table(&mut *conn, cfg, "updates_include").await?;

    let source_directory = ".".to_string();
    let uf = UpdateFetcher::new(source_directory, modules_list, cfg);

    let (updated, recent, archived) = uf.update(&mut *conn).await?;
    let info = format!("Containing {} new and {} archived updates.", recent, archived);
    if updated > 0 {
        info!(">> {} database is up-to-date! {}", cfg.DatabaseName, info);
    } else {
        info!(">> Applied {} queries. {}", updated, info);
    }
    Ok(())
}

#[instrument(skip(conn))]
async fn check_update_table<'a, A: sqlx::Acquire<'a, Database = DbDriver>>(
    conn: A,
    cfg: &ExtendedDBInfo<'_, '_>,
    table_name: &str,
) -> Result<(), DatabaseLoaderError> {
    let mut conn = conn.acquire().await?;

    let res = query(&format!("SHOW TABLES LIKE '{}'", table_name)).fetch_optional(&mut *conn).await?;
    if res.is_some() {
        return Ok(());
    }
    warn!("> Table '{}' not exist! Try add based table", table_name);

    let mut f = cfg.base_files_dir();
    f.push(format!("{table_name}.sql"));

    let db_name = cfg.DatabaseName.clone();
    conn.transaction(|tx| Box::pin(async move {
        apply_file(&mut **tx, f).await.map_err(|e| {
            error!(
                "Failed apply file to database {db_name} due to error: {e}! Does the user (named in *.conf) have `INSERT` and `DELETE` privileges on the MySQL server?",
            );
            e
        })?;
        Ok(())
    })).await
}
