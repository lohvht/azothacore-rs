use azothacore_common::configuration::{DatabaseInfo, DatabaseType, DbUpdates};
use azothacore_database::{database_loader::DatabaseLoader, DbDriver};
use flagset::FlagSet;
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use tokio::sync::Semaphore;

fn default_db_info() -> DatabaseInfo {
    DatabaseInfo {
        Address:      "localhost:8893".to_string(),
        User:         "root".to_string(),
        Password:     "password".to_string(),
        DatabaseName: "".to_string(),
    }
}

pub const DB_NAME_CHARACTERS: &str = "test_azcore_characters";
pub const DB_NAME_HOTFIXES: &str = "test_azcore_hotfixes";
pub const DB_NAME_WORLD: &str = "test_azcore_world";
pub const DB_NAME_AUTH: &str = "test_azcore_auth";

pub async fn test_db_pool_characters(address: Option<String>) -> sqlx::Pool<DbDriver> {
    test_db_pool(address, DB_NAME_CHARACTERS, Some(DatabaseType::Character)).await
}

pub async fn test_db_pool_hotfixes(address: Option<String>) -> sqlx::Pool<DbDriver> {
    test_db_pool(address, DB_NAME_HOTFIXES, Some(DatabaseType::Hotfix)).await
}

pub async fn test_db_pool_world(address: Option<String>) -> sqlx::Pool<DbDriver> {
    test_db_pool(address, DB_NAME_WORLD, Some(DatabaseType::World)).await
}

pub async fn test_db_pool_auth(address: Option<String>) -> sqlx::Pool<DbDriver> {
    test_db_pool(address, DB_NAME_AUTH, Some(DatabaseType::Login)).await
}

pub async fn test_db_pool(address: Option<String>, database_name: &str, setup: Option<DatabaseType>) -> sqlx::Pool<DbDriver> {
    let address = if let Some(a) = address { a } else { default_db_info().Address };
    let default_db_info = DatabaseInfo {
        Address: address,
        DatabaseName: database_name.to_string(),
        ..default_db_info()
    };
    let should_setup = setup.is_some();
    let db_type = if let Some(db_type) = setup { db_type } else { DatabaseType::Login };
    let loader = DatabaseLoader::new(
        db_type,
        default_db_info,
        DbUpdates {
            EnableDatabases: FlagSet::full(),
            AutoSetup: true,
            ..Default::default()
        },
        vec![],
    );
    if should_setup {
        loader.load().await.unwrap()
    } else {
        loader.open_database().await.unwrap()
    }
}

pub fn random_alpanum(n: usize) -> String {
    OsRng.sample_iter(Alphanumeric).take(n).map(char::from).collect()
}

pub static SHARED_TEST_DB_PERMITS: Semaphore = Semaphore::const_new(1);
