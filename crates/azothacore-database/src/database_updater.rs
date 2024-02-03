use std::path::Path;

use sqlx::Connection;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument, warn};
use walkdir::WalkDir;

use crate::{
    database_loader_utils::{apply_file, DatabaseLoaderError},
    database_update_fetcher::UpdateFetcher,
    query,
    DbAcquire,
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

#[instrument(skip_all, fields(base_files_dir=%base_files_dir.as_ref().display()))]
pub async fn db_updater_populate<'a, A: DbAcquire<'a>, P: AsRef<Path>>(
    cancel_token: CancellationToken,
    conn: A,
    base_files_dir: P,
) -> Result<(), DatabaseLoaderError> {
    let mut conn = conn.acquire().await?;

    let res = query("SHOW TABLES").fetch_optional(&mut *conn).await?;
    if res.is_some() {
        return Ok(());
    }
    info!("database is empty, auto populating it from directory...");

    let dir_path = base_files_dir.as_ref();
    if !dir_path.is_dir() {
        let path = format!("{}", dir_path.display());
        error!(">> Directory '{path}' not exist");
        return Err(DatabaseLoaderError::NoBaseDirToPopulate { path });
    }
    let files = WalkDir::new(&dir_path)
        .sort_by(|a, b| a.path().cmp(b.path()))
        .into_iter()
        .filter_map(|e| {
            let e = e.ok()?;
            let p = e.path();
            let file_name = p.file_name()?;
            if file_name.as_encoded_bytes().ends_with(b"sql") || file_name.as_encoded_bytes().ends_with(b"sql.gz") {
                Some(e)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if files.is_empty() {
        let path = format!("{}", dir_path.display());
        error!(">> In directory \"{path}\" not exist '*.sql' files");
        return Err(DatabaseLoaderError::NoBaseDirToPopulate { path });
    }

    for f in files {
        let cancel_token = cancel_token.clone();
        conn.transaction(|tx| {
            Box::pin(async move {
                let is_gz = f.path().extension().filter(|ext| *ext == "gz").is_some();
                apply_file(cancel_token, &mut **tx, f.path(), is_gz).await?;
                Ok::<_, DatabaseLoaderError>(())
            })
        })
        .await?;
    }
    info!(">> Done!\n");
    Ok(())
}

#[instrument(skip(conn))]
pub async fn db_updater_update<'a, A: DbAcquire<'a>>(
    cancel_token: CancellationToken,
    conn: A,
    cfg: &ExtendedDBInfo<'_, '_>,
    modules_list: &[&str],
) -> Result<(), DatabaseLoaderError> {
    let mut conn = conn.acquire().await?;

    info!("Updating {} database...", cfg.DatabaseName);

    check_update_table(cancel_token.clone(), &mut *conn, cfg, "updates").await?;
    check_update_table(cancel_token.clone(), &mut *conn, cfg, "updates_include").await?;

    let uf = UpdateFetcher::new(cancel_token, modules_list, cfg);

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
async fn check_update_table<'a, A: DbAcquire<'a>>(
    cancel_token: CancellationToken,
    conn: A,
    cfg: &ExtendedDBInfo<'_, '_>,
    table_name: &str,
) -> Result<(), DatabaseLoaderError> {
    let mut conn = conn.acquire().await?;

    let res = query(&format!("SHOW TABLES LIKE '{}'", table_name)).fetch_optional(&mut *conn).await?;
    if res.is_some() {
        return Ok(());
    }
    warn!("> Table '{}' not exist! Trying adding base table", table_name);

    let mut is_gz = false;
    let mut f = cfg.base_files_dir().join("{table_name}.sql");
    if !f.exists() {
        f = cfg.base_files_dir().join("{table_name}.sql.gz");
        is_gz = true;
    }

    let db_name = cfg.DatabaseName.clone();
    conn.transaction(|tx| Box::pin(async move {
        apply_file(cancel_token, &mut **tx, f, is_gz).await.map_err(|e| {
            error!(
                "Failed apply file to database {db_name} due to error: {e}! Does the user (named in *.conf) have `INSERT` and `DELETE` privileges on the MySQL server?",
            );
            e
        })?;
        Ok(())
    })).await
}
