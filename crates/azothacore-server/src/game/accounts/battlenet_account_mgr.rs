use azothacore_common::hex_str;
use azothacore_database::{
    database_env::{LoginDatabase, LoginPreparedStmts},
    params,
    DbAcquire,
    DbExecutor,
};
use sha2::{Digest, Sha256};
use tracing::{error, warn};

use super::{
    account_mgr::{AccountMgr, MAX_PASS_STR},
    AccountOpError,
    AccountOpResult,
};

pub const MAX_BNET_EMAIL_STR: usize = 320;

pub struct BattlenetAccountMgr;

impl BattlenetAccountMgr {
    pub async fn create_battlenet_account(email: &str, password: &str, create_game_account: bool) -> AccountOpResult<Option<String>> {
        Self::create_battlenet_account_inner(&LoginDatabase::get(), email, password, create_game_account).await
    }

    pub async fn create_battlenet_account_inner<'a, A: DbAcquire<'a>>(
        login_db: A,
        email: &str,
        password: &str,
        create_game_account: bool,
    ) -> AccountOpResult<Option<String>> {
        if email.len() > MAX_BNET_EMAIL_STR {
            return Err(AccountOpError::NameTooLong);
        }

        if password.len() > MAX_PASS_STR {
            return Err(AccountOpError::PassTooLong);
        }

        let email = email.to_ascii_uppercase();
        let password = password.to_ascii_uppercase();
        let mut login_db = login_db.acquire().await?;

        if Self::get_id_inner(&mut *login_db, &email).await?.is_some() {
            return Err(AccountOpError::NameAlreadyExist);
        }

        let pass_hash = Self::calculate_sha_pass_hash(&email, &password);
        if let Err(e) = LoginDatabase::ins_bnet_account(&mut *login_db, params!(&email, pass_hash)).await {
            warn!(target:"sql::sql", cause=%e, "error when creating bnet account from DB");
            return Err(e.into());
        }

        let new_account_id = Self::get_id_inner(&mut *login_db, &email)
            .await?
            .unwrap_or_else(|| panic!("expect new account to exist after successful insert: {email}"));

        let mut game_account_name = None;
        if create_game_account {
            let bnet_index = 1;
            let account_name = format!("{new_account_id}#{bnet_index}");
            AccountMgr::create_account_inner(&mut *login_db, &account_name, &password, &email, Some((new_account_id, bnet_index))).await?;
            game_account_name = Some(account_name);
        }

        Ok(game_account_name)
    }

    pub async fn change_password(account_id: u32, new_password: &str) -> AccountOpResult<()> {
        Self::change_password_inner(&LoginDatabase::get(), account_id, new_password).await
    }

    pub async fn change_password_inner<'a, A: DbAcquire<'a>>(login_db: A, account_id: u32, new_password: &str) -> AccountOpResult<()> {
        let mut login_db = login_db.acquire().await?;
        let Some(username) = Self::get_name_inner(&mut *login_db, account_id).await? else {
            // account doesn't exist
            return Err(AccountOpError::NameNotExist);
        };

        let username = username.to_ascii_uppercase();
        let new_password = new_password.to_ascii_uppercase();

        if new_password.len() > MAX_PASS_STR {
            return Err(AccountOpError::PassTooLong);
        }

        let pass_hash = Self::calculate_sha_pass_hash(&username, &new_password);
        LoginDatabase::upd_bnet_password(&mut *login_db, params!(pass_hash, account_id)).await?;

        Ok(())
    }

    pub async fn check_password(account_id: u32, password: &str) -> bool {
        Self::check_password_inner(&LoginDatabase::get(), account_id, password).await
    }

    pub async fn check_password_inner<'a, A: DbAcquire<'a>>(login_db: A, account_id: u32, password: &str) -> bool {
        let Ok(mut login_db) = login_db.acquire().await else { return false };
        let Some(username) = Self::get_name_inner(&mut *login_db, account_id).await.ok().flatten() else {
            return false;
        };

        let username = username.to_ascii_uppercase();
        let password = password.to_ascii_uppercase();

        let pass_hash = Self::calculate_sha_pass_hash(&username, &password);
        LoginDatabase::sel_bnet_check_password(&mut *login_db, params!(account_id, pass_hash))
            .await
            .ok()
            .flatten()
            .is_some()
    }

    pub async fn link_with_game_account(email: &str, game_account_name: &str) -> AccountOpResult<()> {
        Self::link_with_game_account_inner(&LoginDatabase::get(), email, game_account_name).await
    }

    pub async fn link_with_game_account_inner<'a, A: DbAcquire<'a>>(login_db: A, email: &str, game_account_name: &str) -> AccountOpResult<()> {
        let mut login_db = login_db.acquire().await?;

        let Some(bnetaccount_id) = Self::get_id_inner(&mut *login_db, email).await? else {
            // account doesn't exist
            return Err(AccountOpError::NameNotExist);
        };

        let Some(game_account_id) = AccountMgr::get_id_inner(&mut *login_db, game_account_name).await? else {
            return Err(AccountOpError::NameNotExist);
        };
        if Self::get_id_by_game_account_inner(&mut *login_db, game_account_id).await?.is_some() {
            // got no bnet account to link, bad link
            return Err(AccountOpError::AccountBadLink);
        }

        let Some(max_index) = Self::get_max_index_inner(&mut *login_db, bnetaccount_id).await? else {
            error!(target:"sql::sql", bnetaccount_id=bnetaccount_id, "get max index failed for Battlenet account, this should not happen will be treated as a DB error");
            return Err(AccountOpError::DbInternalError(sqlx::Error::RowNotFound.to_string()));
        };

        let next_index_to_use = max_index + 1;

        LoginDatabase::upd_bnet_game_account_link(&mut *login_db, params!(bnetaccount_id, next_index_to_use, game_account_id)).await?;

        Ok(())
    }

    pub async fn unlink_game_account(game_account_name: &str) -> AccountOpResult<()> {
        Self::unlink_game_account_inner(&LoginDatabase::get(), game_account_name).await
    }

    pub async fn unlink_game_account_inner<'a, A: DbAcquire<'a>>(login_db: A, game_account_name: &str) -> AccountOpResult<()> {
        let mut login_db = login_db.acquire().await?;
        let Some(game_account_id) = AccountMgr::get_id_inner(&mut *login_db, game_account_name).await? else {
            // account doesn't exist
            return Err(AccountOpError::NameNotExist);
        };

        if Self::get_id_by_game_account_inner(&mut *login_db, game_account_id).await?.is_none() {
            // got no bnet account to unlinked, bad link
            return Err(AccountOpError::AccountBadLink);
        }

        LoginDatabase::upd_bnet_game_account_link(&mut *login_db, params!(None::<u32>, None::<u8>, game_account_id)).await?;

        Ok(())
    }

    pub async fn get_id(username: &str) -> AccountOpResult<Option<u32>> {
        Self::get_id_inner(&LoginDatabase::get(), username).await
    }

    pub async fn get_id_inner<'e, E: DbExecutor<'e>>(login_db: E, username: &str) -> AccountOpResult<Option<u32>> {
        let id = LoginDatabase::sel_bnet_account_id_by_email::<_, (u32,)>(login_db, params!(username)).await?;

        Ok(id.map(|v| v.0))
    }

    pub async fn get_name(account_id: u32) -> AccountOpResult<Option<String>> {
        Self::get_name_inner(&LoginDatabase::get(), account_id).await
    }

    pub async fn get_name_inner<'e, E: DbExecutor<'e>>(login_db: E, account_id: u32) -> AccountOpResult<Option<String>> {
        let name = LoginDatabase::sel_bnet_account_email_by_id::<_, (String,)>(login_db, params!(account_id)).await?;
        Ok(name.map(|v| v.0))
    }

    pub async fn get_id_by_game_account(game_account_id: u32) -> AccountOpResult<Option<u32>> {
        Self::get_id_by_game_account_inner(&LoginDatabase::get(), game_account_id).await
    }

    pub async fn get_id_by_game_account_inner<'e, E: DbExecutor<'e>>(login_db: E, game_account_id: u32) -> AccountOpResult<Option<u32>> {
        let id = LoginDatabase::sel_bnet_account_id_by_game_account::<_, (Option<u32>,)>(login_db, params!(game_account_id)).await?;
        Ok(id.and_then(|v| v.0))
    }

    pub async fn get_max_index(account_id: u32) -> AccountOpResult<Option<u8>> {
        Self::get_max_index_inner(&LoginDatabase::get(), account_id).await
    }

    pub async fn get_max_index_inner<'e, E: DbExecutor<'e>>(login_db: E, account_id: u32) -> AccountOpResult<Option<u8>> {
        let max_index = LoginDatabase::sel_bnet_max_account_index::<_, (Option<u8>,)>(login_db, params!(account_id)).await?;
        Ok(max_index.map(|v| if let Some(i) = v.0 { i } else { 0 }))
    }

    pub fn calculate_sha_pass_hash(name: &str, password: &str) -> String {
        let mut email = Sha256::new();
        email.update(name.as_bytes());
        let email_bytes = &email.finalize()[..];

        let mut sha = Sha256::new();
        sha.update(hex_str!(email_bytes).as_bytes());
        sha.update(":");
        sha.update(password.as_bytes());
        let sha_bytes = &mut sha.finalize()[..];

        sha_bytes.reverse();
        hex_str!(sha_bytes)
    }
}

#[cfg(test)]
mod tests {
    use azothacore_database::database_env::SHARED_TEST_DB_PERMITS;

    use super::*;

    #[tokio::test]
    async fn it_creates_bnet_account() {
        // let _wg = azothacore_common::log::init_console();
        let pool = LoginDatabase::get();

        for (email, password, create_game_account, is_ok) in [
            ("mail@example.com", "abcdefghi", false, Ok(())),
            ("mail@example.com", "abcdefghi", true, Ok(())),
            (&"a".repeat(MAX_BNET_EMAIL_STR + 1), "abcdefghi", true, Err(AccountOpError::NameTooLong)),
            ("mail@example.com", &"a".repeat(MAX_PASS_STR + 1), true, Err(AccountOpError::PassTooLong)),
        ] {
            let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
            let mut txn = pool.begin().await.unwrap();

            let got = BattlenetAccountMgr::create_battlenet_account_inner(&mut *txn, email, password, create_game_account).await;
            let has_game_account = match (got, is_ok) {
                (Err(got_e), Err(expected_e)) => {
                    assert_eq!(got_e, expected_e);
                    continue;
                },
                (Ok(o), Ok(_)) => o,
                (got, expected) => {
                    panic!("left {got:?} != right {expected:?}");
                },
            };
            let new_bnet_account_id = BattlenetAccountMgr::get_id_inner(&mut *txn, email).await.unwrap().unwrap();
            if let Some(got) = &has_game_account {
                let new_game_account_name = format!("{new_bnet_account_id}#1");
                assert_eq!(*got, new_game_account_name);
                let new_game_account_id = AccountMgr::get_id_inner(&mut *txn, &new_game_account_name).await.unwrap().unwrap();
                let got_bnet_account = BattlenetAccountMgr::get_id_by_game_account_inner(&mut *txn, new_game_account_id)
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(got_bnet_account, new_bnet_account_id);
                assert!(AccountMgr::check_password_inner(&mut *txn, new_game_account_id, password).await);
                let max_id = BattlenetAccountMgr::get_max_index_inner(&mut *txn, new_bnet_account_id).await.unwrap().unwrap();
                assert_eq!(max_id, 1);
            }
            let got_email = BattlenetAccountMgr::get_name_inner(&mut *txn, new_bnet_account_id).await.unwrap().unwrap();
            assert_eq!(got_email, email.to_ascii_uppercase());

            assert!(BattlenetAccountMgr::check_password_inner(&mut *txn, new_bnet_account_id, password).await);
        }
    }

    #[tokio::test]
    async fn it_checks_for_correct_password_before_and_after_password_change() {
        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
        // let _wg = azothacore_common::log::init_console();
        let mut txn = LoginDatabase::get().begin().await.unwrap();

        let email = "example@example.com";
        let old_password = "abc1234";
        let new_password = "def5678";
        let invalid_password = "a".repeat(MAX_PASS_STR + 1);

        BattlenetAccountMgr::create_battlenet_account_inner(&mut *txn, email, old_password, false)
            .await
            .unwrap();
        let bnet_account_id = BattlenetAccountMgr::get_id_inner(&mut *txn, email).await.unwrap().unwrap();

        assert!(BattlenetAccountMgr::check_password_inner(&mut *txn, bnet_account_id, old_password).await);
        assert_eq!(
            BattlenetAccountMgr::change_password_inner(&mut *txn, bnet_account_id, &invalid_password).await,
            Err(AccountOpError::PassTooLong)
        );
        BattlenetAccountMgr::change_password_inner(&mut *txn, bnet_account_id, new_password)
            .await
            .unwrap();
        assert!(BattlenetAccountMgr::check_password_inner(&mut *txn, bnet_account_id, new_password).await);
    }

    #[tokio::test]
    async fn it_does_not_change_password_as_no_bnet_account_exists() {
        assert_eq!(BattlenetAccountMgr::change_password(9999, "1234").await, Err(AccountOpError::NameNotExist));
    }

    #[tokio::test]
    async fn it_does_not_check_password_as_no_bnet_account_exists() {
        assert!(!BattlenetAccountMgr::check_password(9999, "1234").await);
    }

    #[tokio::test]
    async fn it_is_able_to_attach_and_detach_game_account() {
        assert_eq!(
            BattlenetAccountMgr::link_with_game_account("non_existent_bnet_email@example.com", "non_existent_game_account").await,
            Err(AccountOpError::NameNotExist)
        );
        assert_eq!(
            BattlenetAccountMgr::unlink_game_account("non_existent_game_account").await,
            Err(AccountOpError::NameNotExist)
        );

        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
        // let _wg = azothacore_common::log::init_console();
        let mut txn = LoginDatabase::get().begin().await.unwrap();

        let bnet_email = "example@example.com";
        // Make un-linked bnet account
        BattlenetAccountMgr::create_battlenet_account_inner(&mut *txn, bnet_email, "1234", false)
            .await
            .unwrap();
        assert_eq!(
            BattlenetAccountMgr::link_with_game_account_inner(&mut *txn, bnet_email, "non_existent_game_account").await,
            Err(AccountOpError::NameNotExist)
        );
        // Make un-linked account
        let ga = "game_account";
        AccountMgr::create_account_inner(&mut *txn, ga, "1234", bnet_email, None).await.unwrap();
        assert_eq!(
            BattlenetAccountMgr::unlink_game_account_inner(&mut *txn, ga).await,
            Err(AccountOpError::AccountBadLink)
        );

        // Link game account
        BattlenetAccountMgr::link_with_game_account_inner(&mut *txn, bnet_email, ga).await.unwrap();
        // Can't link again
        assert_eq!(
            BattlenetAccountMgr::link_with_game_account_inner(&mut *txn, bnet_email, ga).await,
            Err(AccountOpError::AccountBadLink)
        );

        // Unlink game account
        BattlenetAccountMgr::unlink_game_account_inner(&mut *txn, ga).await.unwrap();
        // Can't unlink again
        assert_eq!(
            BattlenetAccountMgr::unlink_game_account_inner(&mut *txn, ga).await,
            Err(AccountOpError::AccountBadLink)
        );
    }
}
