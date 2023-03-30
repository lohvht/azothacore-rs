use std::{fs, io, path::Path};

use owo_colors::OwoColorize;
use sql_parse::{Spanned, Statement::InsertReplace};
use sqlx::MySql;
use thiserror::Error;
use tracing::{error, info};

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
/// HACKFIX: hiro@19/03/2023: sqlx doesnt really play well with mysql style comments
/// Specifically ones that are generated via mysqldump:
/// https://dev.mysql.com/doc/refman/8.0/en/comments.html
/// i.e. comments like this following: `/*!50110 KEY_BLOCK_SIZE=1024 */;`. This function
/// will just swap the positions of the last few tokens `*/;` to `;*/` instead.
pub async fn apply_file<P: AsRef<Path>>(pool: &sqlx::Pool<MySql>, f: P) -> Result<(), DatabaseLoaderError> {
    info!(">> Applying \'{}\'...", f.as_ref().display());

    let file_data = fs::read_to_string(f.as_ref()).map_err(|e| DatabaseLoaderError::OpenApplyFile {
        file:  f.as_ref().to_string_lossy().to_string(),
        inner: e,
    })?;
    let mut tx = pool.begin().await?;
    let mut issues = Vec::new();
    let ast = sql_parse::parse_statements(
        &file_data,
        &mut issues,
        &sql_parse::ParseOptions::new()
            .dialect(sql_parse::SQLDialect::MariaDB)
            .arguments(sql_parse::SQLArguments::None),
    );
    for p in ast {
        let p_span = if let InsertReplace(insert_rep) = p {
            // TODO: Report this error for span for insert replace here
            // The generated range doesnt include the last char in `sql_parse` crate
            let s = insert_rep.span();
            s.start..s.end + 1
        } else {
            p.span()
        };
        let stmt_str = &file_data[p_span];

        let res = sqlx::query(stmt_str).execute(&mut tx).await;
        if let Err(e) = res {
            error!(
                "Applying of file \'{}\' to database failed!\n\
                  If you are a user, please pull the latest revision from the repository.\n\
                  Also make sure you have not applied any of the databases with your sql client.\n\
                  You cannot use auto-update system and import sql files from the repository with your sql client. \n\
                  If you are a developer, please fix your sql query.\nstatement applied was:\n{}\n",
                f.as_ref().display(),
                stmt_str,
            );
            tx.rollback().await?;
            if !issues.is_empty() {
                error!("there were additional potential issues with queries");
                let mut i = 0;
                for is in &issues {
                    i += 1;
                    let iss_span = &file_data[is.span.to_owned()];
                    error!(">> issue {}: spanned => {}, verbose_issue => {:?}", i, iss_span.blue(), is.green());
                }
            }
            return Err(e.into());
        }
    }

    tx.commit().await?;
    Ok(())
}
