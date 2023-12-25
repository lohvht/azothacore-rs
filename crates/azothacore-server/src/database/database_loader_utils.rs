use std::{fs, io, path::Path};

use sqlx::{Connection, Executor};
use thiserror::Error;
use tracing::{error, info};

use crate::database::DbExecutor;

#[derive(Error, Debug)]
pub enum DatabaseLoaderError {
    #[error("Database pool was NOT open. There were errors opening the connection or errors with the underlying driver.")]
    SqlxGeneralError(#[from] sqlx::Error),
    #[error("database specific error")]
    DatabaseSpecific(#[from] Box<dyn sqlx::error::DatabaseError>),
    #[error("Directory '{path}' not exist or path does not have any files to populate")]
    NoBaseDirToPopulate { path: String },
    #[error("unable to open file to apply or update: {file}")]
    OpenApplyFile {
        file:  String,
        #[source]
        inner: io::Error,
    },
    #[error("generic error: {msg}")]
    Generic { msg: String },
}

/// Applies the file's content to the given pool.
pub async fn apply_file<'e, P: AsRef<Path>, E: DbExecutor<'e>>(conn: E, f: P) -> Result<(), DatabaseLoaderError> {
    let file_path = f.as_ref().display();
    info!(">> Applying \'{}\'...", f.as_ref().display());

    let file_data = fs::read_to_string(f.as_ref()).map_err(|e| DatabaseLoaderError::OpenApplyFile {
        file:  file_path.to_string(),
        inner: e,
    })?;

    let file_path = file_path.to_string();
    // NOTE: hirogoro@21dec2023: Raw unprepared execution, by not enclosing with sqlx::query function
    // => See: https://github.com/launchbadge/sqlx/issues/2557
    // enclosing in sqlx::query tells sqlx to treat the statement as a prepared stmt.
    _ = conn.execute(file_data.as_str()).await.map_err(|e| {
        error!(
            r#"Applying of file '{file_path}' to database failed!
          If you are a user, please pull the latest revision from the repository.
          Also make sure you have not applied any of the databases with your sql client.
          You cannot use auto-update system and import sql files from the repository with your sql client.
          If you are a developer, please fix your sql query. Err was:
          
          {e}"#,
        );
        e
    })?;
    Ok(())
}
