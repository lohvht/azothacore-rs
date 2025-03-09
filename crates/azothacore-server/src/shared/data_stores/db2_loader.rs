use std::{collections::BTreeMap, io};

use azothacore_database::{DbDriver, DbExecutor};
use futures::TryStreamExt;
use sqlx::{query, query_as, Database, FromRow};
use tracing::warn;
use wow_db2::{raw_localised_strs_record_from_sql_row, wdc1::FileLoader, DB2};

pub type DB2FileLoader<D> = FileLoader<D>;

pub struct DB2DatabaseLoader;

impl DB2DatabaseLoader {
    pub async fn load<'e, E: DbExecutor<'e>, D: DB2 + for<'r> FromRow<'r, <DbDriver as Database>::Row> + Send + Unpin>(
        hotfix_db: E,
        db2_data: &mut BTreeMap<u32, D>,
    ) -> io::Result<()> {
        let mut res = query_as::<_, D>(D::db2_sql_stmt()).fetch(hotfix_db);
        while let Some(db2_entry) = res
            .try_next()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("error querying for db2: {e}")))?
        {
            let entry = db2_data.entry(db2_entry.id()).or_default();
            // do a total replacement
            *entry = db2_entry;
        }

        Ok(())
    }

    pub async fn load_localised_strings<'e, E: DbExecutor<'e>, D: DB2>(hotfix_db: E, db2_data: &mut BTreeMap<u32, D>) -> io::Result<()> {
        let Some(locale_stmt) = D::db2_sql_locale_stmt() else { return Ok(()) };
        let mut res = query(locale_stmt).fetch(hotfix_db);

        let db2_fields = D::db2_fields();
        while let Some(row) = res
            .try_next()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("error querying localised strings for db2: {e}")))?
        {
            let str_entry = raw_localised_strs_record_from_sql_row(&db2_fields, &row)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("error mapping localised string row to db2 entry: err={e}")))?;
            let Some(db2) = db2_data.get_mut(&str_entry.id) else {
                warn!(
                    query_stmt = locale_stmt,
                    "unexpected ID {id} found in locale table for DB2 {db2_name}. Please check the locale table again",
                    id = str_entry.id,
                    db2_name = D::db2_file()
                );
                continue;
            };
            db2.merge_strs(&str_entry);
        }
        Ok(())
    }
}
