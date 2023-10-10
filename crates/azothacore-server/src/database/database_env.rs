use std::sync::OnceLock;

use sqlx::{MySql, Pool};

mod login_db {
    use hugsqlx::HugSqlx;

    #[derive(HugSqlx)]
    #[queries = "sql/queries/login_db_prep_stmts.sql"]
    pub struct LoginDatabase {}
}

mod character_db {
    use hugsqlx::HugSqlx;

    #[derive(HugSqlx)]
    #[queries = "sql/queries/character_db_prep_stmts.sql"]
    pub struct CharacterDatabase {}
}

mod world_db {
    use hugsqlx::HugSqlx;

    #[derive(HugSqlx)]
    #[queries = "sql/queries/world_db_prep_stmts.sql"]
    pub struct WorldDatabase {}
}

pub use character_db::{CharacterDatabase, HugSql as CharacterPreparedStmts};
pub use login_db::{HugSql as LoginPreparedStmts, LoginDatabase};
pub use world_db::{HugSql as WorldPreparedStmts, WorldDatabase};

impl WorldDatabase {
    pub fn get() -> &'static Pool<MySql> {
        WORLD_DB.get().expect("WorldDatabase is not initialised yet")
    }

    pub fn set(pool: Pool<MySql>) {
        WORLD_DB.set(pool).expect("WorldDatabase has already been set")
    }
}

impl CharacterDatabase {
    pub fn get() -> &'static Pool<MySql> {
        CHARACTER_DB.get().expect("CharacterDatabase is not initialised yet")
    }

    pub fn set(pool: Pool<MySql>) {
        CHARACTER_DB.set(pool).expect("CharacterDatabase has already been set")
    }
}

impl LoginDatabase {
    pub fn get() -> &'static Pool<MySql> {
        LOGIN_DB.get().expect("LoginDatabase is not initialised yet")
    }

    pub fn set(pool: Pool<MySql>) {
        LOGIN_DB.set(pool).expect("LoginDatabase has already been set")
    }
}

static WORLD_DB: OnceLock<Pool<MySql>> = OnceLock::new();
static CHARACTER_DB: OnceLock<Pool<MySql>> = OnceLock::new();
static LOGIN_DB: OnceLock<Pool<MySql>> = OnceLock::new();
