use std::{
    collections::BTreeMap,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use azothacore_common::{
    configuration::{DatabaseInfo, DatabaseType, DbUpdates},
    hex_str,
};
use sha2::{Digest, Sha256};
use sqlx::{
    mysql::MySqlDatabaseError,
    pool::PoolOptions,
    query,
    query_as,
    query_with,
    types::chrono::{DateTime, Utc},
    Connection,
};
use tokio::time::Instant;
use tracing::{debug, error, info, trace, warn};
use walkdir::WalkDir;

use crate::{
    args,
    database_loader_utils::{apply_file, DatabaseLoaderError},
    DbDriver,
    ExtendedDBInfo,
};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Copy)]
enum FetcherState {
    Released,
    Custom,
    Module,
    Archived,
}

impl FetcherState {
    fn is_custom_update(&self) -> bool {
        matches!(self, FetcherState::Custom | FetcherState::Module)
    }
}

impl FromStr for FetcherState {
    type Err = DatabaseLoaderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RELEASED" => Ok(FetcherState::Released),
            "CUSTOM" => Ok(FetcherState::Custom),
            "MODULE" => Ok(FetcherState::Module),
            "ARCHIVED" => Ok(FetcherState::Archived),
            e => Err(DatabaseLoaderError::Generic {
                msg: format!("invalid fetcher enum: {}", e),
            }),
        }
    }
}

impl Display for FetcherState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            FetcherState::Archived => "ARCHIVED",
            FetcherState::Custom => "CUSTOM",
            FetcherState::Module => "MODULE",
            FetcherState::Released => "RELEASED",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Copy)]
enum UpdateMode {
    Apply,
    Rehash,
}

struct AppliedFileEntry {
    #[expect(dead_code)]
    name:           String,
    hash:           String,
    state:          FetcherState,
    #[expect(dead_code)]
    unix_timestamp: DateTime<Utc>,
}

pub struct DatabaseLoader {
    modules_list:    Vec<String>,
    database_config: ExtendedDBInfo,
}

fn get_sha256_hash<P: AsRef<Path>>(fp: P) -> Result<String, DatabaseLoaderError> {
    let file_content = fs::read_to_string(fp.as_ref())
        .map_err(|e| DatabaseLoaderError::OpenApplyFile {
            file:  fp.as_ref().to_string_lossy().to_string(),
            inner: e,
        })
        .inspect_err(|e| {
            let f = if let DatabaseLoaderError::OpenApplyFile { file, .. } = &e { file } else { "" };
            error!(
                "Failed to open the sql update {} for reading! \n\
                Stopping the server to keep the database integrity, \n\
                try to identify and solve the issue or disable the database updater.",
                f,
            );
        })?;
    let mut hasher = Sha256::new();
    hasher.update(file_content.as_bytes());
    let hash_bytes = &hasher.finalize()[..];
    Ok(hex_str!(hash_bytes))
}

#[derive(PartialEq)]
struct DbFile {
    path:  PathBuf,
    state: FetcherState,
}

impl DbFile {
    fn is_gz(&self) -> bool {
        self.path.extension().filter(|ext| *ext == "gz").is_some()
    }

    fn name(&self) -> String {
        if self.is_gz() { self.path.file_stem() } else { self.path.file_name() }
            .and_then(|f| f.to_str())
            .unwrap()
            .to_string()
    }

    fn hash(&self) -> Result<String, DatabaseLoaderError> {
        get_sha256_hash(&self.path)
    }
}

const DATABASE_FILES_SRC_ROOT: &str = ".";

impl DatabaseLoader {
    pub fn new(db_type: DatabaseType, database_config: DatabaseInfo, update_config: DbUpdates, modules_list: Vec<String>) -> Self {
        Self {
            modules_list,
            database_config: ExtendedDBInfo::new(database_config, update_config, db_type),
        }
    }

    fn module_db_paths(&self) -> Vec<PathBuf> {
        self.modules_list
            .iter()
            .map(|e| {
                let mut p = Path::new(DATABASE_FILES_SRC_ROOT).to_path_buf();

                p.extend(&["azothacore-modules", e, "data/sql", self.database_config.db_module_name()]);
                p
            })
            .collect()
    }

    /// Loads and prepares the database as required. it first opens the connection to the DB
    /// Then populates the DB if needed and keeps it up to date.
    pub async fn load(self) -> Result<sqlx::Pool<DbDriver>, DatabaseLoaderError> {
        if !self.database_config.updates_enabled() {
            info!("Automatic database updates are disabled for {}", self.database_config.DatabaseName);
        }
        let pool = self.open_database().await?;
        if let Err(e) = self.populate_database(pool.clone()).await {
            pool.close().await;
            return Err(e);
        }
        if let Err(e) = self.update_database(pool.clone()).await {
            pool.close().await;
            return Err(e);
        }
        Ok(pool)
    }

    pub async fn open_database(&self) -> Result<sqlx::Pool<DbDriver>, DatabaseLoaderError> {
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
                    warn!("Database {} does not exist!", self.database_config.DatabaseName);
                    info!("Creating database '{}'...", self.database_config.DatabaseName);
                    query(&format!(
                        "CREATE DATABASE `{}` DEFAULT CHARACTER SET UTF8MB4 COLLATE utf8mb4_general_ci",
                        self.database_config.DatabaseName
                    ))
                    .execute(&conn)
                    .await?;
                    info!("Done.\n");
                    conn.close().await;
                }
                PoolOptions::<DbDriver>::new().connect(&self.database_config.connect_url()).await?
            },
            Err(e) => return Err(e.into()),
        };
        Ok(pool)
    }

    async fn populate_database(&self, pool: sqlx::Pool<DbDriver>) -> Result<(), DatabaseLoaderError> {
        if !self.database_config.updates_enabled() {
            return Ok(());
        }
        let res = query("SHOW TABLES").fetch_optional(&pool).await?;
        if res.is_some() {
            return Ok(());
        }
        info!("database is empty, auto populating it from directory...");
        let dir_path = self.database_config.base_files_dir();
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
            pool.acquire()
                .await?
                .transaction(|tx| {
                    Box::pin(async move {
                        let is_gz = f.path().extension().filter(|ext| *ext == "gz").is_some();
                        apply_file(&mut **tx, f.path(), is_gz).await?;
                        Ok::<_, DatabaseLoaderError>(())
                    })
                })
                .await?;
        }
        info!(">> Done!\n");

        Ok(())
    }

    async fn update_database(&self, pool: sqlx::Pool<DbDriver>) -> Result<(), DatabaseLoaderError> {
        if !self.database_config.updates_enabled() {
            return Ok(());
        }
        info!("Updating {} database...", self.database_config.DatabaseName);

        self.check_update_table(pool.clone(), "updates").await?;
        self.check_update_table(pool.clone(), "updates_include").await?;

        let (updated, recent, archived) = self.fetch_and_apply_updates(pool).await?;
        let info = format!("Containing {} new and {} archived updates.", recent, archived);
        if updated > 0 {
            info!(">> {} database is up-to-date! {}", self.database_config.DatabaseName, info);
        } else {
            info!(">> Applied {} queries. {}", updated, info);
        }

        Ok(())
    }

    async fn fetch_and_apply_updates(&self, pool: sqlx::Pool<DbDriver>) -> Result<(usize, usize, usize), DatabaseLoaderError> {
        let available = self.get_file_list(pool.clone()).await?;
        if available.is_empty() {
            return Ok((0, 0, 0));
        }
        let mut applied = Self::receive_applied_files(pool.clone()).await?;
        let start: (usize, usize) = (0, 0);
        // Count updates
        let (count_recent_updates, count_archived_updates) = applied.iter().fold(start, |acc: (usize, usize), e| {
            if let FetcherState::Released = e.1.state {
                (acc.0 + 1, acc.1)
            } else {
                (acc.0, acc.1 + 1)
            }
        });

        let hash_to_filename: BTreeMap<_, _> = applied.iter().map(|e| (e.1.hash.clone(), e.0.clone())).collect();

        let mut imported_updates = 0;

        let mut new_available = Vec::with_capacity(available.len());
        let mut custom = vec![];
        for f in available {
            if !f.state.is_custom_update() {
                new_available.push(f);
            } else {
                custom.push(f);
            }
        }
        new_available.append(&mut custom);

        for f in new_available.iter() {
            let Some(mode) = self
                .check_if_update_should_be_applied(pool.clone(), &mut applied, &hash_to_filename, &new_available, f)
                .await?
            else {
                continue;
            };

            self.apply_update_file(pool.clone(), f, mode).await?;
            applied.remove(&f.name());
            imported_updates += 1;
        }
        let do_cleanup = self.database_config.updates.should_cleanup(applied.len());
        // Cleanup up orphaned entries (if enabled)
        let mut to_cleanup = vec![];
        for (filename, file_entry) in applied {
            if file_entry.state == FetcherState::Module {
                continue;
            }
            warn!(">> The file \'{filename}\' was applied to the database, but is missing in your update directory now!");
            if do_cleanup {
                info!("Deleting orphaned entry from \'updates\' table in DB: \'{filename}\'...");
                to_cleanup.push(filename);
            }
        }
        if do_cleanup {
            Self::clean_up(pool.clone(), to_cleanup).await?;
        } else if !to_cleanup.is_empty() {
            error!(
                "Cleanup is disabled! There were {} dirty files applied to your database, but they are now missing in your source directory!",
                to_cleanup.len()
            );
        }
        Ok((imported_updates, count_recent_updates, count_archived_updates))
    }

    async fn get_file_list(&self, pool: sqlx::Pool<DbDriver>) -> Result<Vec<DbFile>, DatabaseLoaderError> {
        const MAX_DEPTH: usize = 10;

        let directories = self.receive_included_directories(pool).await?;

        let mut storage: Vec<DbFile> = Vec::new();
        for (dir_path, dir_state) in directories {
            let files = WalkDir::new(dir_path)
                .sort_by(|a, b| a.path().cmp(b.path()))
                .max_depth(MAX_DEPTH)
                .into_iter()
                .filter_map(|e| {
                    let e = e.ok()?;
                    if "sql" != e.path().extension()? {
                        return None;
                    }
                    Some(e)
                });

            for f in files {
                trace!("Added locale file \"{}\" state '{dir_state:?}'.", f.path().display());
                let r = DbFile {
                    path:  f.path().to_path_buf(),
                    state: dir_state,
                };
                if storage.contains(&r) {
                    return Err(DatabaseLoaderError::Generic { msg: format!("Updating failed due to duplicate filename \"{}\" found. Because updates are ordered by their filenames, every name needs to be unique!", r.path.display()) });
                }
                storage.push(r);
            }
        }

        Ok(storage)
    }

    async fn receive_included_directories(&self, pool: sqlx::Pool<DbDriver>) -> Result<Vec<(PathBuf, FetcherState)>, DatabaseLoaderError> {
        let mut directories: Vec<(PathBuf, FetcherState)> = query_as::<_, (String, String)>("SELECT `path`, `state` FROM `updates_include`")
            .fetch_all(&pool)
            .await?
            .into_iter()
            .filter_map(|(p, state)| {
                let p = if p.starts_with("$/") {
                    let mut pb = Path::new(DATABASE_FILES_SRC_ROOT).to_path_buf();
                    pb.push(p.trim_start_matches("$/"));
                    pb
                } else {
                    Path::new(&p).to_path_buf()
                };
                if !p.is_dir() {
                    warn!(
                        "DBUpdater: Given update include directory \"{}\" does not exist or isn't a directory, skipped!",
                        p.to_string_lossy(),
                    );
                    return None;
                }

                let s: FetcherState = FetcherState::from_str(&state)
                    .map_err(|e| {
                        warn!(
                            "DBUpdater: Given update include directory \"{}\" has invalid state, error was {}, skipped!",
                            p.to_string_lossy(),
                            e,
                        );
                        e
                    })
                    .ok()?;

                Some((p, s))
            })
            .collect();

        directories.extend(self.module_db_paths().into_iter().filter_map(|p| {
            if !p.is_dir() {
                warn!(
                    "DBUpdater: Given module directory \"{}\" does not exist or isn't a directory, skipped!",
                    p.to_string_lossy(),
                );
                return None;
            }
            debug!("Added applied modules file \"{}\" from remote.", p.to_string_lossy());
            Some((p, FetcherState::Module))
        }));

        Ok(directories)
    }

    async fn receive_applied_files(pool: sqlx::Pool<DbDriver>) -> Result<BTreeMap<String, AppliedFileEntry>, DatabaseLoaderError> {
        let map = query_as::<_, (String, String, String, DateTime<Utc>)>("SELECT `name`, `hash`, `state`, `timestamp` FROM `updates` ORDER BY `name` ASC")
            .fetch_all(&pool)
            .await?
            .into_iter()
            .filter_map(|(name, hash, state, unix_timestamp)| {
                let state = FetcherState::from_str(&state)
                    .map_err(|e| {
                        warn!(
                            "DBUpdater: update from `updates` table with name \"{}\" has invalid state, error was {}, skipped!",
                            name, e,
                        );
                        e
                    })
                    .ok()?;

                let e = AppliedFileEntry {
                    name: name.clone(),
                    hash,
                    state,
                    unix_timestamp,
                };

                Some((name, e))
            })
            .collect();
        Ok(map)
    }

    async fn check_if_update_should_be_applied(
        &self,
        pool: sqlx::Pool<DbDriver>,
        applied: &mut BTreeMap<String, AppliedFileEntry>,
        applied_hash_to_name: &BTreeMap<String, String>,
        available: &[DbFile],
        file: &DbFile,
    ) -> Result<Option<UpdateMode>, DatabaseLoaderError> {
        let hash = file.hash()?;
        info!("Checking update \"{}\"...", file.name());

        let mut mode = UpdateMode::Apply;
        match (applied.get(&file.name()), applied_hash_to_name.get(&hash)) {
            (Some(_), _) if !self.database_config.updates.Redundancy => {
                // If redundancy is disabled, skip it, because the update is already applied.
                info!(">> Update is already applied, skipping redundancy checks.");
                applied.remove(&file.name());
                return Ok(None);
            },
            (Some(iter), _)
                if !self.database_config.updates.ArchivedRedundancy && iter.state == FetcherState::Archived && file.state == FetcherState::Archived =>
            {
                // If the update is in an archived directory and is marked as archived in our database, skip redundancy checks (archived updates never change).
                info!(">> Update is archived and marked as archived in database, skipping redundancy checks.");
                applied.remove(&file.name());
                return Ok(None);
            },
            (Some(iter), _) if self.database_config.updates.AllowRehash && iter.hash.is_empty() => {
                mode = UpdateMode::Rehash;
                info!(">> Re-hashing update \"{}\" \'{}\'...", file.path.to_string_lossy(), &hash);
            },
            (Some(iter), _) if iter.hash != hash => {
                info!(
                    ">> Reapplying update \"{}\" \'{}\' -> \'{}\' (it changed)...",
                    file.path.to_string_lossy(),
                    iter.hash,
                    hash
                );
            },
            (Some(iter), _) => {
                if iter.state != file.state {
                    info!(">> Updating the state of \"{}\" to \'{:?}\'...", file.path.to_string_lossy(), file.state);
                    query_with("UPDATE `updates` SET `state` = ? where `name` = ?", args!(file.state.to_string(), file.name())?)
                        .execute(&pool)
                        .await?;
                }
                info!(">> Update is already applied and matches the hash \'{}\'.", hash);
                applied.remove(&file.name());
                return Ok(None);
            },
            // Update is not in our applied list
            (None, Some(applied_hash_filename)) => {
                // Catch renames (different filename, but same hash)
                // Check if the original file was removed. If not, we've got a problem.
                // Push localeIter forward
                let available_local = available.iter().find(|e| e.name() == *applied_hash_filename);
                let Some(available_local_file) = available_local else {
                    // It is safe to treat the file as renamed here
                    let from = applied_hash_filename;
                    let to = file.name();
                    info!(">> Renaming update \"{from}\" to \"{to}\" \'{hash}\'.");
                    let mut txn = pool.begin().await?;
                    query("DELETE FROM `updates` WHERE `name`= ?").bind(&to).execute(&mut *txn).await?;
                    query_with("UPDATE `updates` SET `name`=? WHERE `name`=?", args!(&to, &from)?)
                        .execute(&mut *txn)
                        .await?;
                    txn.commit().await?;
                    applied.remove(applied_hash_filename);
                    return Ok(None);
                };
                // Conflict!
                warn!(
                    ">> It seems like the update \"{}\" \'{}\' was renamed, but the old file is still there! \n\
                        Treating it as a new file! (It is probably an unmodified copy of the file \"{}\")",
                    file.path.to_string_lossy(),
                    hash,
                    available_local_file.path.display(),
                );
            },
            (None, None) => {
                // Apply the update if it was never seen before.
                info!(">> Applying update \"{}\" \'{}\'...", file.path.to_string_lossy(), hash);
            },
        };
        Ok(Some(mode))
    }

    async fn apply_update_file(&self, pool: sqlx::Pool<DbDriver>, file: &DbFile, mode: UpdateMode) -> Result<(), DatabaseLoaderError> {
        let mut txn = pool.begin().await?;
        let now = Instant::now();
        if matches!(mode, UpdateMode::Apply) {
            apply_file(&mut *txn, &file.path, file.is_gz()).await?;
        }
        let speed = now.elapsed();
        query_with(
            "REPLACE INTO `updates` (`name`, `hash`, `state`, `speed`) VALUES (?,?,?,?)",
            args!(file.name(), file.hash()?, file.state.to_string(), speed.as_millis().to_string())?,
        )
        .execute(&mut *txn)
        .await?;
        txn.commit().await?;
        Ok(())
    }

    async fn clean_up(pool: sqlx::Pool<DbDriver>, storage: Vec<String>) -> Result<(), DatabaseLoaderError> {
        if storage.is_empty() {
            return Ok(());
        }
        let q = format!("DELETE FROM `updates` WHERE `name` IN ({})", vec!["?"; storage.len()].join(","));
        let mut q = query(q.as_str());
        for name in storage {
            q = q.bind(name);
        }
        q.execute(&pool).await?;
        Ok(())
    }

    async fn check_update_table(&self, pool: sqlx::Pool<DbDriver>, table_name: &str) -> Result<(), DatabaseLoaderError> {
        let res = query(&format!("SHOW TABLES LIKE '{}'", table_name)).fetch_optional(&pool).await?;
        if res.is_some() {
            return Ok(());
        }
        warn!("> Table '{}' not exist! Trying adding base table", table_name);

        let mut is_gz = false;
        let mut f = self.database_config.base_files_dir().join("{table_name}.sql");
        if !f.exists() {
            f = self.database_config.base_files_dir().join("{table_name}.sql.gz");
            is_gz = true;
        }

        let db_name = self.database_config.DatabaseName.clone();
        let mut txn = pool.begin().await?;
        apply_file(&mut *txn, f, is_gz).await.map_err(|e| {
            error!(
                "Failed apply file to database {db_name} due to error: {e}! Does the user (named in *.conf) have `INSERT` and `DELETE` privileges on the MySQL server?",
            );
            e
        })?;
        txn.commit().await?;
        Ok(())
    }
}
