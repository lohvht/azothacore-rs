use sqlx::Pool;

mod login_db {
    use hugsqlx::HugSqlx;

    #[derive(HugSqlx)]
    #[queries = "data/sql/queries/login_db_prep_stmts.sql"]
    pub struct LoginDatabase {}
}

mod character_db {
    use hugsqlx::HugSqlx;

    #[derive(HugSqlx)]
    #[queries = "data/sql/queries/character_db_prep_stmts.sql"]
    pub struct CharacterDatabase {}
}

mod world_db {
    use hugsqlx::HugSqlx;

    #[derive(HugSqlx)]
    #[queries = "data/sql/queries/world_db_prep_stmts.sql"]
    pub struct WorldDatabase {}
}

mod hotfix_db {
    use hugsqlx::HugSqlx;

    #[derive(HugSqlx)]
    #[queries = "data/sql/queries/hotfix_db_prep_stmts.sql"]
    pub struct HotfixDatabase {}
}

pub use character_db::{CharacterDatabase, HugSql as CharacterPreparedStmts};
pub use hotfix_db::{HotfixDatabase, HugSql as HotfixPreparedStmts};
pub use login_db::{HugSql as LoginPreparedStmts, LoginDatabase};
pub use world_db::{HugSql as WorldPreparedStmts, WorldDatabase};

use crate::DbDriver;

#[cfg(not(feature = "test-utils"))]
impl WorldDatabase {
    pub async fn close() {
        if let Some(pool) = WORLD_DB.get() {
            pool.close().await;
        }
    }

    pub fn get() -> Pool<DbDriver> {
        WORLD_DB.get().expect("WorldDatabase is not initialised yet").clone()
    }

    pub fn set(pool: Pool<DbDriver>) {
        WORLD_DB.set(pool).expect("WorldDatabase has already been set")
    }
}

#[cfg(not(feature = "test-utils"))]
impl CharacterDatabase {
    pub async fn close() {
        if let Some(pool) = CHARACTER_DB.get() {
            pool.close().await;
        }
    }

    pub fn get() -> Pool<DbDriver> {
        CHARACTER_DB.get().expect("CharacterDatabase is not initialised yet").clone()
    }

    pub fn set(pool: Pool<DbDriver>) {
        CHARACTER_DB.set(pool).expect("CharacterDatabase has already been set")
    }
}

#[cfg(not(feature = "test-utils"))]
impl LoginDatabase {
    pub async fn close() {
        if let Some(pool) = LOGIN_DB.get() {
            pool.close().await;
        }
    }

    pub fn get() -> Pool<DbDriver> {
        LOGIN_DB.get().expect("LoginDatabase is not initialised yet").clone()
    }

    pub fn set(pool: Pool<DbDriver>) {
        LOGIN_DB.set(pool).expect("LoginDatabase has already been set")
    }
}

#[cfg(not(feature = "test-utils"))]
impl HotfixDatabase {
    pub async fn close() {
        if let Some(pool) = HOTFIX_DB.get() {
            pool.close().await;
        }
    }

    pub fn get() -> Pool<DbDriver> {
        HOTFIX_DB.get().expect("HotfixDatabase is not initialised yet").clone()
    }

    pub fn set(pool: Pool<DbDriver>) {
        HOTFIX_DB.set(pool).expect("HotfixDatabase has already been set")
    }
}

#[cfg(not(feature = "test-utils"))]
static WORLD_DB: std::sync::OnceLock<Pool<DbDriver>> = std::sync::OnceLock::new();
#[cfg(not(feature = "test-utils"))]
static CHARACTER_DB: std::sync::OnceLock<Pool<DbDriver>> = std::sync::OnceLock::new();
#[cfg(not(feature = "test-utils"))]
static LOGIN_DB: std::sync::OnceLock<Pool<DbDriver>> = std::sync::OnceLock::new();
#[cfg(not(feature = "test-utils"))]
static HOTFIX_DB: std::sync::OnceLock<Pool<DbDriver>> = std::sync::OnceLock::new();

#[cfg(feature = "test-utils")]
pub static SHARED_TEST_DB_PERMITS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

#[cfg(feature = "test-utils")]
fn get_test_pool(typ: azothacore_common::configuration::DatabaseType) -> Pool<DbDriver> {
    dotenvy::from_filename(".test.env").ok();
    let env_var = match typ {
        azothacore_common::configuration::DatabaseType::Character => "CHARACTER_DATABASE_URL",
        azothacore_common::configuration::DatabaseType::Hotfix => "HOTFIX_DATABASE_URL",
        azothacore_common::configuration::DatabaseType::World => "WORLD_DATABASE_URL",
        azothacore_common::configuration::DatabaseType::Login => "LOGIN_DATABASE_URL",
        a => panic!("Not supported: {a:?}"),
    };
    let url = dotenvy::var(env_var).unwrap_or_else(|e| panic!("err={e}; env var must be set to run DB tests, check .env.test for {env_var}"));

    sqlx::pool::PoolOptions::<DbDriver>::new()
        .max_connections(5)
        .idle_timeout(Some(::std::time::Duration::from_secs(30)))
        .connect_lazy(&url)
        .unwrap()
}
#[cfg(feature = "test-utils")]
impl WorldDatabase {
    pub async fn close() {
        panic!("Test utils no need to close");
    }

    pub fn set(_pool: Pool<DbDriver>) {
        panic!("Test utils no need to set");
    }

    pub fn get() -> Pool<DbDriver> {
        get_test_pool(azothacore_common::configuration::DatabaseType::World)
    }
}
#[cfg(feature = "test-utils")]
impl CharacterDatabase {
    pub async fn close() {
        panic!("Test utils no need to close");
    }

    pub fn set(_pool: Pool<DbDriver>) {
        panic!("Test utils no need to set");
    }

    pub fn get() -> Pool<DbDriver> {
        get_test_pool(azothacore_common::configuration::DatabaseType::Character)
    }
}
#[cfg(feature = "test-utils")]
impl LoginDatabase {
    pub async fn close() {
        panic!("Test utils no need to close");
    }

    pub fn set(_pool: Pool<DbDriver>) {
        panic!("Test utils no need to set");
    }

    pub fn get() -> Pool<DbDriver> {
        get_test_pool(azothacore_common::configuration::DatabaseType::Login)
    }
}
#[cfg(feature = "test-utils")]
impl HotfixDatabase {
    pub async fn close() {
        panic!("Test utils no need to close");
    }

    pub fn set(_pool: Pool<DbDriver>) {
        panic!("Test utils no need to set");
    }

    pub fn get() -> Pool<DbDriver> {
        get_test_pool(azothacore_common::configuration::DatabaseType::Hotfix)
    }
}
