use std::{
    fs,
    io::{self, Read},
    path::Path,
};

use azothacore_common::utils::buffered_file_open;
use flate2::bufread::GzDecoder;
use thiserror::Error;
use tracing::{error, info};

use crate::DbExecutor;

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

fn map_open_err(file_path: &str) -> impl FnOnce(io::Error) -> DatabaseLoaderError {
    let file = file_path.to_string();
    move |e| DatabaseLoaderError::OpenApplyFile { file, inner: e }
}

/// Applies the file's content to the given pool.
pub async fn apply_file<'e, P: AsRef<Path>, E: DbExecutor<'e>>(conn: E, f: P, is_gz: bool) -> Result<(), DatabaseLoaderError> {
    let file_path = f.as_ref().display().to_string();
    info!(">> Applying \'{file_path}\'...");

    let file_data = if is_gz {
        let r = buffered_file_open(f.as_ref()).map_err(map_open_err(&file_path))?;
        let mut gz = GzDecoder::new(r);
        let mut d = String::new();
        gz.read_to_string(&mut d).map_err(map_open_err(&file_path))?;
        d
    } else {
        fs::read_to_string(f.as_ref()).map_err(map_open_err(&file_path))?
    };

    tokio::select! {
        res = conn.execute(file_data.as_str()) => {
                // NOTE: hirogoro@21dec2023: Raw unprepared execution, by not enclosing with sqlx::query function
                // => See: https://github.com/launchbadge/sqlx/issues/2557
                // enclosing in sqlx::query tells sqlx to treat the statement as a prepared stmt.
            if let Err(e) = &res {
                error!(
                    r#"Applying of file '{file_path}' to database failed!
                  If you are a user, please pull the latest revision from the repository.
                  Also make sure you have not applied any of the databases with your sql client.
                  You cannot use auto-update system and import sql files from the repository with your sql client.
                  If you are a developer, please fix your sql query. Err was:
                  
                  {e}"#,
                );
            }
            res?;
        }
    }
    Ok(())
}
