use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    string::ToString,
    time::Duration,
};

use azothacore_common::{configuration::DbUpdates, hex_str};
use sha2::{Digest, Sha256};
use sqlx::{
    types::chrono::{DateTime, Utc},
    Acquire,
    Row,
};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, trace, warn};
use walkdir::WalkDir;

use crate::{
    database_loader_utils::{apply_file, DatabaseLoaderError},
    params,
    query,
    query_with,
    DbAcquire,
    DbExecutor,
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

impl ToString for FetcherState {
    fn to_string(&self) -> String {
        match *self {
            FetcherState::Archived => "ARCHIVED".into(),
            FetcherState::Custom => "CUSTOM".into(),
            FetcherState::Module => "MODULE".into(),
            FetcherState::Released => "RELEASED".into(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Copy)]
enum UpdateMode {
    Apply,
    Rehash,
}

struct AppliedFileEntry {
    #[expect(dead_code)]
    name:           PathBuf,
    hash:           String,
    state:          FetcherState,
    #[expect(dead_code)]
    unix_timestamp: DateTime<Utc>,
}

pub struct UpdateFetcher<'c, 'l, 'd, 'u> {
    cancel_token: CancellationToken,
    module_list:  &'l [&'l str],
    database_cfg: &'c ExtendedDBInfo<'d, 'u>,
}

const DATABASE_FILES_SRC_ROOT: &str = ".";

impl<'c, 'l, 'd, 'u> UpdateFetcher<'c, 'l, 'd, 'u> {
    pub fn new(cancel_token: CancellationToken, module_list: &'l [&'l str], database_cfg: &'c ExtendedDBInfo<'d, 'u>) -> Self {
        Self {
            cancel_token,
            module_list,
            database_cfg,
        }
    }

    fn module_db_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        let r = self.module_list.iter().map(|e| {
            let mut p = Path::new(DATABASE_FILES_SRC_ROOT).to_path_buf();

            p.extend(&["azothacore-script-modules", e, "data/sql", self.database_cfg.db_module_name()]);
            p
        });
        r
    }

    #[instrument(skip(self, conn))]
    async fn receive_included_directories<'a, A: DbAcquire<'a>>(&self, conn: A) -> Result<impl Iterator<Item = (PathBuf, FetcherState)>, DatabaseLoaderError> {
        let mut conn = conn.acquire().await?;

        let mut directories: Vec<(PathBuf, FetcherState)> = query("SELECT `path`, `state` FROM `updates_include`")
            .fetch_all(&mut *conn)
            .await?
            .iter()
            .filter_map(|row| {
                let p = row.get::<String, _>("path");
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

                let s: FetcherState = FetcherState::from_str(row.get::<String, _>("state").as_str())
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

        directories.extend(self.module_db_paths().filter_map(|p| {
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

        Ok(directories.into_iter())
    }

    // #[instrument(skip(self, pool))]
    async fn get_file_list<'a, A: DbAcquire<'a>>(&self, conn: A) -> Result<Vec<(PathBuf, FetcherState)>, DatabaseLoaderError> {
        let mut conn = conn.acquire().await?;

        const MAX_DEPTH: usize = 10;

        let directories = self.receive_included_directories(&mut *conn).await?;

        let mut storage: Vec<(PathBuf, FetcherState)> = Vec::new();
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
                let r = (f.path().to_path_buf(), dir_state);
                if storage.contains(&r) {
                    let msg = format!("Updating failed due to duplicate filename \"{}\" found. Because updates are ordered by their filenames, every name needs to be unique!", f.path().display());
                    error!("{}", msg);
                    return Err(DatabaseLoaderError::Generic { msg });
                }
                storage.push(r);
            }
        }

        Ok(storage)
    }

    #[instrument(skip(self, conn))]
    async fn receive_applied_files<'a, A: DbAcquire<'a>>(&self, conn: A) -> Result<BTreeMap<PathBuf, AppliedFileEntry>, DatabaseLoaderError> {
        let mut conn = conn.acquire().await?;

        let map = query("SELECT `name`, `hash`, `state`, `timestamp` FROM `updates` ORDER BY `name` ASC")
            .fetch_all(&mut *conn)
            .await?
            .iter()
            .filter_map(|row| {
                let name: String = row.get("name");
                let state = FetcherState::from_str(&row.get::<String, _>("state"))
                    .map_err(|e| {
                        warn!(
                            "DBUpdater: update from `updates` table with name \"{}\" has invalid state, error was {}, skipped!",
                            name, e,
                        );
                        e
                    })
                    .ok()?;

                let e = AppliedFileEntry {
                    name: Path::new(&name).to_path_buf(),
                    hash: row.get("hash"),
                    state,
                    unix_timestamp: row.get("timestamp"),
                };

                Some((Path::new(&name).to_path_buf(), e))
            })
            .collect();
        Ok(map)
    }

    #[instrument(skip(self, conn))]
    pub async fn update<'a, A: DbAcquire<'a>>(&self, conn: A) -> Result<(usize, usize, usize), DatabaseLoaderError> {
        let mut conn = conn.acquire().await?;

        let available = self.get_file_list(&mut *conn).await?;
        if available.is_empty() {
            return Ok((0, 0, 0));
        }
        let mut applied = self.receive_applied_files(&mut *conn).await?;
        let start: (usize, usize) = (0, 0);
        // Count updates
        let (count_recent_updates, count_archived_updates) = applied.iter().fold(start, |acc: (usize, usize), e| {
            if let FetcherState::Released = e.1.state {
                (acc.0 + 1, acc.1)
            } else {
                (acc.0, acc.1 + 1)
            }
        });

        let hash_to_filename: BTreeMap<String, PathBuf> = applied.iter().map(|e| (e.1.hash.clone(), e.0.clone())).collect();

        let mut imported_updates = 0;

        for (avail_file, avail_file_state) in available.iter() {
            if !avail_file_state.is_custom_update() {
                imported_updates += apply_update_file(
                    self.cancel_token.clone(),
                    self.database_cfg.updates,
                    &mut *conn,
                    &mut applied,
                    &hash_to_filename,
                    &available,
                    avail_file,
                    avail_file_state,
                )
                .await?;
            }
        }
        for (avail_file, avail_file_state) in available.iter() {
            if avail_file_state.is_custom_update() {
                imported_updates += apply_update_file(
                    self.cancel_token.clone(),
                    self.database_cfg.updates,
                    &mut *conn,
                    &mut applied,
                    &hash_to_filename,
                    &available,
                    avail_file,
                    avail_file_state,
                )
                .await?;
            }
        }
        let do_cleanup = self.database_cfg.updates.should_cleanup(applied.len());
        // Cleanup up orphaned entries (if enabled)
        let mut to_cleanup = vec![];
        for (filename, file_entry) in applied {
            let filename_str = format!("{}", filename.display());
            if file_entry.state == FetcherState::Module {
                continue;
            }
            warn!(">> The file \'{filename_str}\' was applied to the database, but is missing in your update directory now!");
            to_cleanup.push(filename_str);
            if !do_cleanup {
                continue;
            }
            info!("Deleting orphaned entry from \'updates\' table in DB: \'{}\'...", filename.display());
        }
        if do_cleanup {
            clean_up(&mut *conn, to_cleanup).await?;
        } else if !to_cleanup.is_empty() {
            error!(
                "Cleanup is disabled! There were {} dirty files applied to your database, but they are now missing in your source directory!",
                to_cleanup.len()
            );
        }
        Ok((imported_updates, count_recent_updates, count_archived_updates))
    }
}

fn get_sha256_hash<P: AsRef<Path>>(fp: P) -> Result<String, DatabaseLoaderError> {
    let file_content = fs::read_to_string(fp.as_ref())
        .map_err(|e| DatabaseLoaderError::OpenApplyFile {
            file:  fp.as_ref().to_string_lossy().to_string(),
            inner: e,
        })
        .map_err(|e| {
            let f = if let DatabaseLoaderError::OpenApplyFile { file, .. } = &e { file } else { "" };
            error!(
                "Failed to open the sql update {} for reading! \n\
                Stopping the server to keep the database integrity, \n\
                try to identify and solve the issue or disable the database updater.",
                f,
            );
            e
        })?;
    let mut hasher = Sha256::new();
    hasher.update(file_content.as_bytes());
    let hash_bytes = &hasher.finalize()[..];
    Ok(hex_str!(hash_bytes))
}

#[instrument(skip(pool))]
async fn update_state<'e, E>(pool: E, file_name: &str, state: FetcherState) -> Result<(), DatabaseLoaderError>
where
    E: DbExecutor<'e>,
{
    query("UPDATE `updates` SET `state` = ? where `name` = ?").execute(pool).await?;
    Ok(())
}

#[expect(clippy::too_many_arguments)]
#[instrument(skip_all, fields(
    file_path=%file_path.display(),
    file_state=?file_state,
    update_cfg=?update_cfg,
))]
async fn apply_update_file<'a, A: DbAcquire<'a>>(
    cancel_token: CancellationToken,
    update_cfg: &DbUpdates,
    conn: A,
    applied: &mut BTreeMap<PathBuf, AppliedFileEntry>,
    applied_hash_to_filename: &BTreeMap<String, PathBuf>,
    available: &[(PathBuf, FetcherState)],
    file_path: &PathBuf,
    file_state: &FetcherState,
) -> Result<usize, DatabaseLoaderError> {
    let mut conn = conn.acquire().await?;

    let is_gz = file_path.extension().filter(|ext| *ext == "gz").is_some();
    let filename = if is_gz { file_path.file_stem() } else { file_path.file_name() }
        .and_then(|f| f.to_str())
        .unwrap();

    info!("Checking update \"{}\"...", filename);

    if let Some(iter) = applied.get(&PathBuf::from(filename)) {
        // If redundancy is disabled, skip it, because the update is already applied.
        if !update_cfg.Redundancy {
            info!(">> Update is already applied, skipping redundancy checks.");
            applied.remove(&PathBuf::from(filename));
            return Ok(0);
        }
        // If the update is in an archived directory and is marked as archived in our database, skip redundancy checks (archived updates never change).
        if !update_cfg.ArchivedRedundancy && iter.state == FetcherState::Archived && file_state == &FetcherState::Archived {
            info!(">> Update is archived and marked as archived in database, skipping redundancy checks.");
            applied.remove(&PathBuf::from(filename));
            return Ok(0);
        }
    }
    let hash = get_sha256_hash(file_path)?;

    let mut mode = UpdateMode::Apply;

    if let Some(iter) = applied.get(&PathBuf::from(filename)) {
        if update_cfg.AllowRehash && iter.hash.is_empty() {
            mode = UpdateMode::Rehash;
            info!(">> Re-hashing update \"{}\" \'{}\'...", file_path.to_string_lossy(), &hash);
        } else if iter.hash != hash {
            info!(
                ">> Reapplying update \"{}\" \'{}\' -> \'{}\' (it changed)...",
                file_path.to_string_lossy(),
                iter.hash,
                hash
            );
        } else {
            if iter.state != *file_state {
                info!(">> Updating the state of \"{}\" to \'{:?}\'...", file_path.to_string_lossy(), file_state,);
                update_state(&mut *conn, filename, *file_state).await?
            }
            info!(">> Update is already applied and matches the hash \'{}\'.", hash);
            applied.remove(&PathBuf::from(filename));
            return Ok(0);
        }
    } else {
        // Update is not in our applied list
        // Catch renames (different filename, but same hash)
        let hash_found = applied_hash_to_filename.get(&hash);
        if let Some(applied_hash_filename) = hash_found {
            // Check if the original file was removed. If not, we've got a problem.
            // Push localeIter forward
            let available_local = available.iter().find(|e| {
                let available_filename = &e.0;
                available_filename == applied_hash_filename
            });
            // Conflict!
            if let Some((available_local_filename, _)) = available_local {
                warn!(
                    ">> It seems like the update \"{}\" \'{}\' was renamed, but the old file is still there! \n\
                    Treating it as a new file! (It is probably an unmodified copy of the file \"{}\")",
                    file_path.to_string_lossy(),
                    hash,
                    available_local_filename.to_string_lossy(),
                );
            } else {
                // It is safe to treat the file as renamed here
                info!(
                    ">> Renaming update \"{}\" to \"{}\" \'{}\'.",
                    applied_hash_filename.to_string_lossy(),
                    file_path.to_string_lossy(),
                    hash,
                );

                rename_entry(&mut *conn, &applied_hash_filename.to_string_lossy(), filename).await?;
                applied.remove(applied_hash_filename);
                return Ok(0);
            }
        } else {
            // Apply the update if it was never seen before.
            info!(">> Applying update \"{}\" \'{}\'...", file_path.to_string_lossy(), hash,);
        }
    }

    let mut tx = conn.begin().await?;
    let now = Instant::now();
    match mode {
        UpdateMode::Apply => {
            apply_file(cancel_token, &mut *tx, file_path, is_gz).await?;
            let speed = now.elapsed();
            update_entry(&mut *tx, filename, &hash, file_state, speed).await?;
        },
        UpdateMode::Rehash => {
            let speed = now.elapsed();
            update_entry(&mut *tx, filename, &hash, file_state, speed).await?;
        },
    }
    tx.commit().await?;

    applied.remove(&PathBuf::from(filename));

    Ok(if let UpdateMode::Apply = mode { 1 } else { 0 })
}

#[instrument(skip(conn))]
async fn rename_entry<'a, A: DbAcquire<'a>>(conn: A, from: &str, to: &str) -> Result<(), DatabaseLoaderError> {
    let mut conn = conn.acquire().await?;
    query("DELETE FROM `updates` WHERE `name`= ?").bind(to).execute(&mut *conn).await?;
    query_with("UPDATE `updates` SET `name`=? WHERE `name`=?", params!(to, from))
        .execute(&mut *conn)
        .await?;
    Ok(())
}

#[instrument(skip(e))]
async fn update_entry<'e, E: DbExecutor<'e>>(e: E, filename: &str, hash: &str, state: &FetcherState, speed: Duration) -> Result<(), DatabaseLoaderError> {
    query_with(
        "REPLACE INTO `updates` (`name`, `hash`, `state`, `speed`) VALUES (?,?,?,?)",
        params!(filename, hash, state.to_string(), speed.as_millis().to_string()),
    )
    .execute(e)
    .await?;
    Ok(())
}

#[instrument(skip(e))]
async fn clean_up<'e, E: DbExecutor<'e>>(e: E, storage: Vec<String>) -> Result<(), DatabaseLoaderError> {
    if storage.is_empty() {
        return Ok(());
    }
    let q = format!("DELETE FROM `updates` WHERE `name` IN ({})", vec!["?"; storage.len()].join(","));
    let mut q = query(q.as_str());
    for name in storage {
        q = q.bind(name);
    }
    q.execute(e).await?;
    Ok(())
}
