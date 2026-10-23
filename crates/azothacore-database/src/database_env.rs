mod login_db {
    #![allow(dead_code)]

    use hugsqlx::HugSqlx;

    #[derive(HugSqlx)]
    #[queries = "data/sql/queries/login_db_prep_stmts.sql"]
    pub struct LoginStmts {}
}

mod character_db {
    #![allow(dead_code)]

    use hugsqlx::HugSqlx;

    #[derive(HugSqlx)]
    #[queries = "data/sql/queries/character_db_prep_stmts.sql"]
    pub struct CharacterStmts {}
}

mod world_db {
    #![allow(dead_code)]

    use hugsqlx::HugSqlx;

    #[derive(HugSqlx)]
    #[queries = "data/sql/queries/world_db_prep_stmts.sql"]
    pub struct WorldStmts {}
}

mod hotfix_db {
    #![allow(dead_code)]

    use hugsqlx::HugSqlx;

    #[derive(HugSqlx)]
    #[queries = "data/sql/queries/hotfix_db_prep_stmts.sql"]
    pub struct HotfixStmts {}
}

use azothacore_common::deref_boilerplate;
use bevy::prelude::*;
pub use character_db::HugSql as CharacterPreparedStmts;
pub use hotfix_db::HugSql as HotfixPreparedStmts;
pub use login_db::HugSql as LoginPreparedStmts;
use sqlx::Pool;
pub use world_db::HugSql as WorldPreparedStmts;

use crate::DbDriver;

#[derive(Resource, Clone)]
pub struct CharacterDatabase(pub Pool<DbDriver>);
deref_boilerplate!(CharacterDatabase, Pool<DbDriver>, 0);
impl CharacterPreparedStmts for CharacterDatabase {}

#[derive(Resource, Clone)]
pub struct HotfixDatabase(pub Pool<DbDriver>);
deref_boilerplate!(HotfixDatabase, Pool<DbDriver>, 0);
impl HotfixPreparedStmts for HotfixDatabase {}

#[derive(Resource, Clone)]
pub struct LoginDatabase(pub Pool<DbDriver>);
deref_boilerplate!(LoginDatabase, Pool<DbDriver>, 0);
impl LoginPreparedStmts for LoginDatabase {}

#[derive(Resource, Clone)]
pub struct WorldDatabase(pub Pool<DbDriver>);
deref_boilerplate!(WorldDatabase, Pool<DbDriver>, 0);
impl WorldPreparedStmts for WorldDatabase {}
