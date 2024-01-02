use std::sync::OnceLock;

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

impl WorldDatabase {
    pub fn get() -> &'static Pool<DbDriver> {
        WORLD_DB.get().expect("WorldDatabase is not initialised yet")
    }

    pub fn set(pool: Pool<DbDriver>) {
        WORLD_DB.set(pool).expect("WorldDatabase has already been set")
    }
}

impl CharacterDatabase {
    pub fn get() -> &'static Pool<DbDriver> {
        CHARACTER_DB.get().expect("CharacterDatabase is not initialised yet")
    }

    pub fn set(pool: Pool<DbDriver>) {
        CHARACTER_DB.set(pool).expect("CharacterDatabase has already been set")
    }
}

impl LoginDatabase {
    pub fn get() -> &'static Pool<DbDriver> {
        LOGIN_DB.get().expect("LoginDatabase is not initialised yet")
    }

    pub fn set(pool: Pool<DbDriver>) {
        LOGIN_DB.set(pool).expect("LoginDatabase has already been set")
    }
}

impl HotfixDatabase {
    pub fn get() -> &'static Pool<DbDriver> {
        HOTFIX_DB.get().expect("HotfixDatabase is not initialised yet")
    }

    pub fn set(pool: Pool<DbDriver>) {
        HOTFIX_DB.set(pool).expect("HotfixDatabase has already been set")
    }
}

static WORLD_DB: OnceLock<Pool<DbDriver>> = OnceLock::new();
static CHARACTER_DB: OnceLock<Pool<DbDriver>> = OnceLock::new();
static LOGIN_DB: OnceLock<Pool<DbDriver>> = OnceLock::new();
static HOTFIX_DB: OnceLock<Pool<DbDriver>> = OnceLock::new();
