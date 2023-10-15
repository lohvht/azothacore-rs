use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    string::ToString,
    time::Duration,
};

use azothacore_common::configuration::DbUpdates;
use hex_fmt::HexFmt;
use sha2::{Digest, Sha256};
use sqlx::{MySql, Row};
use tokio::time::Instant;
use tracing::{debug, error, info, instrument, trace, warn};
use walkdir::WalkDir;

use crate::database::{
    database_loader_utils::{apply_file, DatabaseLoaderError},
    qargs,
    sql,
    sql_w_args,
};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Copy)]
enum FetcherState {
    Released,
    Custom,
    Module,
    Archived,
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
    unix_timestamp: u64,
}

pub struct UpdateFetcher<'u> {
    src_directory: String,
    module_target: String,
    module_list:   BTreeSet<String>,
    update_cfg:    &'u DbUpdates,
}

impl<'u> UpdateFetcher<'u> {
    pub fn new<Iter: IntoIterator<Item = String>>(
        src_directory: String,
        module_target: String,
        module_iterator: Iter,
        update_cfg: &'u DbUpdates,
    ) -> Self {
        let mut module_list = BTreeSet::new();
        module_list.extend(module_iterator);
        Self {
            src_directory,
            module_target,
            module_list,
            update_cfg,
        }
    }

    fn module_path_iterator(&self) -> impl Iterator<Item = PathBuf> + '_ {
        let r = self.module_list.iter().map(|e| {
            let mut p = Path::new(&self.src_directory).to_path_buf();
            p.extend(&["src", "modules", e, "data/sql", &self.module_target]);
            p
        });
        r
    }

    #[instrument(skip(self, pool))]
    async fn receive_included_directories(
        &self,
        pool: &sqlx::Pool<MySql>,
    ) -> Result<impl Iterator<Item = (PathBuf, FetcherState)>, DatabaseLoaderError> {
        let mut directories: Vec<(PathBuf, FetcherState)> = sql("SELECT `path`, `state` FROM `updates_include`")
            .fetch_all(pool)
            .await?
            .iter()
            .filter_map(|row| {
                let p = row.get::<String, _>("path");
                let p = if p.starts_with('$') {
                    let mut pb = Path::new(&self.src_directory).to_path_buf();
                    pb.push(p.trim_start_matches('$'));
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

        directories.extend(self.module_path_iterator().filter_map(|p| {
            if !p.is_dir() {
                warn!(
                    "DBUpdater: Given module directory \"{}\" does not exist or isn't a directory, skipped!",
                    p.to_string_lossy(),
                );
                return None;
            }
            trace!("Added applied modules file \"{}\" from remote.", p.to_string_lossy());
            Some((p, FetcherState::Module))
        }));

        Ok(directories.into_iter())
    }

    // #[instrument(skip(self, pool))]
    async fn get_file_list(&self, pool: &sqlx::Pool<MySql>) -> Result<Vec<(PathBuf, FetcherState)>, DatabaseLoaderError> {
        const MAX_DEPTH: usize = 10;

        let directories = self.receive_included_directories(pool).await?;

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
                let r = (f.path().to_path_buf(), dir_state);
                trace!("Added locale file \"{}\" state '{:?}'.", r.0.to_string_lossy(), r.1);
                if storage.contains(&r) {
                    let msg = format!("Updating failed due to duplicate filename \"{}\" found. Because updates are ordered by their filenames, every name needs to be unique!", r.0.to_string_lossy());
                    error!("{}", msg);
                    return Err(DatabaseLoaderError::Generic { msg });
                }
                storage.push(r);
            }
        }

        Ok(storage)
    }

    #[instrument(skip(self, pool))]
    async fn receive_applied_files(&self, pool: &sqlx::Pool<MySql>) -> Result<BTreeMap<PathBuf, AppliedFileEntry>, DatabaseLoaderError> {
        let map: BTreeMap<PathBuf, AppliedFileEntry> =
            sql("SELECT `name`, `hash`, `state`, UNIX_TIMESTAMP(`timestamp`) as `unix_timestamp` FROM `updates` ORDER BY `name` ASC")
                .fetch_all(pool)
                .await?
                .iter()
                .filter_map(|row| {
                    let name: String = row.get("name");
                    let state: FetcherState = FetcherState::from_str(row.get::<String, _>("state").as_str())
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
                        unix_timestamp: row.get("unix_timestamp"),
                    };

                    Some((Path::new(&name).to_path_buf(), e))
                })
                .collect();
        Ok(map)
    }

    #[instrument(skip(self, pool))]
    pub async fn update(&self, pool: &sqlx::Pool<MySql>) -> Result<(usize, usize, usize), DatabaseLoaderError> {
        let available = self.get_file_list(pool).await?;
        if available.is_empty() {
            return Ok((0, 0, 0));
        }
        let mut applied = self.receive_applied_files(pool).await?;
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

        // Apply default updates
        for (avail_file, avail_file_state) in available.iter() {
            if !matches!(avail_file_state, FetcherState::Custom | FetcherState::Module) {
                imported_updates += apply_update_file(
                    self.update_cfg,
                    pool,
                    &mut applied,
                    &hash_to_filename,
                    &available,
                    avail_file,
                    avail_file_state,
                )
                .await?;
            }
        }
        // Apply only custom/module updates
        for (avail_file, avail_file_state) in available.iter() {
            if matches!(avail_file_state, FetcherState::Custom | FetcherState::Module) {
                imported_updates += apply_update_file(
                    self.update_cfg,
                    pool,
                    &mut applied,
                    &hash_to_filename,
                    &available,
                    avail_file,
                    avail_file_state,
                )
                .await?;
            }
        }
        // Cleanup up orphaned entries (if enabled)
        if !applied.is_empty() {
            let do_cleanup =
                self.update_cfg.CleanDeadRefMaxCount.is_none() || applied.len() <= self.update_cfg.CleanDeadRefMaxCount.unwrap();
            let to_cleanup = applied
                .into_iter()
                .filter_map(|entry| {
                    if entry.1.state == FetcherState::Module {
                        return None;
                    }
                    warn!(
                        ">> The file \'{}\' was applied to the database, but is missing in your update directory now!",
                        entry.0.to_string_lossy(),
                    );
                    if !do_cleanup {
                        return None;
                    }
                    info!("Deleting orphaned entry \'{}\'...", entry.0.to_string_lossy());
                    Some(entry.0.to_string_lossy().to_string())
                })
                .collect::<Vec<_>>();
            if !to_cleanup.is_empty() {
                if do_cleanup {
                    clean_up(pool, to_cleanup).await?;
                } else {
                    error!(
                        "Cleanup is disabled! There were {} dirty files applied to your database, but they are now missing in your source directory!",
                        to_cleanup.len()
                    );
                }
            }
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
            let f = if let DatabaseLoaderError::OpenApplyFile { file, .. } = &e {
                file
            } else {
                ""
            };
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
    Ok(format!("{}", HexFmt(hash_bytes)))
}

#[instrument(skip(pool))]
async fn update_state(pool: &sqlx::Pool<MySql>, file_name: String, state: FetcherState) -> Result<(), DatabaseLoaderError> {
    sql("UPDATE `updates` SET `state` = ? where `name` = ?").execute(pool).await?;
    Ok(())
}

#[instrument(skip_all, fields(
    file_path=format!("{}", file_path.display()),
    file_state=format!("{file_state:?}"),
    redundancy_checks=update_cfg.Redundancy,
    archived_redundancy=update_cfg.ArchivedRedundancy,
    allow_rehash=update_cfg.AllowRehash,
))]
async fn apply_update_file(
    update_cfg: &DbUpdates,
    pool: &sqlx::Pool<MySql>,
    applied: &mut BTreeMap<PathBuf, AppliedFileEntry>,
    applied_hash_to_filename: &BTreeMap<String, PathBuf>,
    available: &[(PathBuf, FetcherState)],
    file_path: &PathBuf,
    file_state: &FetcherState,
) -> Result<usize, DatabaseLoaderError> {
    debug!("Checking update \"{}\"...", file_path.to_string_lossy());

    if let Some(iter) = applied.get(file_path) {
        // If redundancy is disabled, skip it, because the update is already applied.
        if !update_cfg.Redundancy {
            debug!(">> Update is already applied, skipping redundancy checks.");
            applied.remove(file_path);
            return Ok(0);
        }
        // If the update is in an archived directory and is marked as archived in our database, skip redundancy checks (archived updates never change).
        if !update_cfg.ArchivedRedundancy && iter.state == FetcherState::Archived && file_state == &FetcherState::Archived {
            debug!(">> Update is archived and marked as archived in database, skipping redundancy checks.");
            applied.remove(file_path);
            return Ok(0);
        }
    }
    let hash = get_sha256_hash(file_path)?;

    let mut mode = UpdateMode::Apply;

    if let Some(iter) = applied.get(file_path) {
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
                debug!(
                    ">> Updating the state of \"{}\" to \'{:?}\'...",
                    file_path.to_string_lossy(),
                    file_state,
                );
                update_state(pool, file_path.to_string_lossy().to_string(), *file_state).await?
            }
            debug!(">> Update is already applied and matches the hash \'{}\'.", hash);
            applied.remove(file_path);
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

                rename_entry(pool, &applied_hash_filename.to_string_lossy(), &file_path.to_string_lossy()).await?;
                applied.remove(applied_hash_filename);
                return Ok(0);
            }
        } else {
            // Apply the update if it was never seen before.
            info!(">> Applying update \"{}\" \'{}\'...", file_path.to_string_lossy(), hash,);
        }
    }

    let now = Instant::now();
    match mode {
        UpdateMode::Apply => {
            apply_file(pool, file_path).await?;
            let speed = now.elapsed();
            update_entry(pool, &file_path.to_string_lossy(), &hash, file_state, speed).await?;
        },
        UpdateMode::Rehash => {
            let speed = now.elapsed();
            update_entry(pool, &file_path.to_string_lossy(), &hash, file_state, speed).await?;
        },
    }

    applied.remove(file_path);

    Ok(if let UpdateMode::Apply = mode { 1 } else { 0 })
}

#[instrument(skip(pool))]
async fn rename_entry(pool: &sqlx::Pool<MySql>, from: &str, to: &str) -> Result<(), DatabaseLoaderError> {
    sql("DELETE FROM `updates` WHERE `name`= ?").bind(to).execute(pool).await?;
    sql_w_args("UPDATE `updates` SET `name`=? WHERE `name`=?", qargs!(to, from))
        .execute(pool)
        .await?;
    Ok(())
}

#[instrument(skip(pool))]
async fn update_entry(
    pool: &sqlx::Pool<MySql>,
    filename: &str,
    hash: &str,
    state: &FetcherState,
    speed: Duration,
) -> Result<(), DatabaseLoaderError> {
    sql_w_args(
        "REPLACE INTO `updates` (`name`, `hash`, `state`, `speed`) VALUES (?,?,?,?)",
        qargs!(filename, hash, state.to_string(), speed.as_millis().to_string()),
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[instrument(skip(pool))]
async fn clean_up(pool: &sqlx::Pool<MySql>, storage: Vec<String>) -> Result<(), DatabaseLoaderError> {
    let q = format!("DELETE FROM `updates` WHERE `name` IN ({})", vec!["?"; storage.len()].join(","));
    let mut q = sql(q.as_str());
    for name in storage {
        q = q.bind(name);
    }
    q.execute(pool).await?;
    Ok(())
}
