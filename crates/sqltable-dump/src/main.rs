use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
    thread::sleep,
    time::Duration,
};

use flate2::{write::GzEncoder, Compression};
use sqlx::{mysql::MySqlPoolOptions, Executor, Row};
use tokio::task::JoinSet;

async fn database_names(user: &str, password: &str, host: &str) -> Vec<String> {
    let url = format!("mysql://{user}:{password}@{host}");
    println!("Retrieving dbnames from {url}");
    let pool = MySqlPoolOptions::new().connect(&url).await.unwrap();
    pool.fetch_all("SHOW DATABASES").await.unwrap().into_iter().map(|row| row.get(0)).collect()
}

async fn export_db(user: &str, password: &str, host: &str, db_name: &str) -> Vec<String> {
    let pool = MySqlPoolOptions::new()
        .connect(&format!("mysql://{user}:{password}@{host}/{db_name}"))
        .await
        .unwrap();

    let tables: Vec<String> = pool.fetch_all("SHOW TABLES").await.unwrap().into_iter().map(|row| row.get(0)).collect();
    tables
}

// Upper limit in bytes
const UPPER_LIMIT: usize = 1024 * 1024 * 50;

/// SQL Table Dump is an easy way to dump all tables available from a given MySQL address.
fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    let user = std::env::var("DBUSER").ok().unwrap_or("azcore".into());
    let password = std::env::var("DBPASSWORD").ok().unwrap_or("azcore".into());
    let host = std::env::var("DBHOST").ok().unwrap_or("127.0.0.1".into());
    let output_dir = std::env::var("OUTPUT_DIR").ok().unwrap_or("data/sql/base".into());

    let rt_handler = rt.handle().clone();
    rt.block_on(async move {
        let dbnames = database_names(&user, &password, &host).await;

        let mut jhs = JoinSet::new();

        for db in dbnames {
            if db == "information_schema" {
                continue;
            }

            let tables = export_db(&user, &password, &host, &db).await;
            // println!("db={db}: tables={tables:?}");

            for table in tables {
                fs::create_dir_all(format!("{output_dir}/{db}")).unwrap();
                let out = format!("{output_dir}/{db}/{table}.sql");
                let out_gz = format!("{output_dir}/{db}/{table}.sql.gz");

                let mut cmd = Command::new("mysqldump");
                cmd.args([
                    &format!("-u{user}"),
                    &format!("-h{host}"),
                    "--protocol=tcp",
                    &format!("-p{password}"),
                    "--skip-comments",
                    "--skip-set-charset",
                    "--routines",
                    "--extended-insert",
                    "--order-by-primary",
                    "--single-transaction",
                    "--quick",
                    &db,
                    &table,
                ])
                .stderr(Stdio::piped())
                .stdout(Stdio::piped());
                let child = cmd.spawn().unwrap();
                jhs.spawn_blocking_on(
                    move || {
                        // the actual writing and gzipping might take a while
                        // we put it inside a blocking async task.
                        let cmd_output = child.wait_with_output().unwrap();
                        if !cmd_output.status.success() {
                            panic!("{}", String::from_utf8_lossy(&cmd_output.stderr));
                        }
                        println!("Writing to {out}");
                        if cmd_output.stdout.len() > UPPER_LIMIT - 1 {
                            // if the output is greater than the upper limit, we may need to zip it
                            // first
                            let mut gz_out = GzEncoder::new(fs::File::create(&out_gz).unwrap(), Compression::best());
                            gz_out.write_all(&cmd_output.stdout).unwrap();
                        } else {
                            let mut outfile = fs::File::create(&out).unwrap();
                            outfile.write_all(&cmd_output.stdout).unwrap();
                        }
                    },
                    &rt_handler,
                );
                sleep(Duration::from_millis(10));
            }
        }
        while let Some(res) = jhs.join_next().await {
            res.unwrap();
        }
    });
}
