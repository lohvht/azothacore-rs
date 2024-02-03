use azothacore_common::hex_str;
use azothacore_database::{
    database_env::{LoginDatabase, LoginPreparedStmts},
    params,
};
use sha2::{Digest, Sha256};
use tracing::{error, warn};

use super::{
    account_mgr::{AccountMgr, MAX_PASS_STR},
    AccountOpError,
    AccountOpResult,
    DbBattlenetAccount,
    DbBnetMaxIndex,
    DbEmail,
    DbId,
};

pub const MAX_BNET_EMAIL_STR: usize = 320;

pub struct BattlenetAccountMgr;

impl BattlenetAccountMgr {
    pub async fn create_battlenet_account(email: &str, password: &str, create_game_account: bool) -> AccountOpResult<Option<String>> {
        if email.len() > MAX_BNET_EMAIL_STR {
            return Err(AccountOpError::NameTooLong);
        }

        if password.len() > MAX_PASS_STR {
            return Err(AccountOpError::PassTooLong);
        }

        let email = email.to_ascii_uppercase();
        let password = password.to_ascii_uppercase();

        if Self::get_id(&email).await?.is_some() {
            return Err(AccountOpError::NameAlreadyExist);
        }

        let login_db = &LoginDatabase::get();
        let pass_hash = Self::calculate_sha_pass_hash(&email, &password);
        if let Err(e) = LoginDatabase::ins_bnet_account(login_db, params!(&email, pass_hash)).await {
            warn!(target:"sql::sql", cause=%e, "error when creating bnet account from DB");
            return Err(e.into());
        }

        let new_account_id = Self::get_id(&email)
            .await?
            .unwrap_or_else(|| panic!("expect new account to exist after successful insert: {email}"));

        let mut game_account_name = None;
        if create_game_account {
            let bnet_index = 1;
            let account_name = format!("{new_account_id}#{bnet_index}");
            AccountMgr::create_account(&account_name, &password, &email, Some((new_account_id, bnet_index))).await?;
            game_account_name = Some(account_name);
        }

        Ok(game_account_name)
    }

    pub async fn change_password(account_id: u32, new_password: &str) -> AccountOpResult<()> {
        let Some(username) = Self::get_name(account_id).await? else {
            // account doesn't exist
            return Err(AccountOpError::NameNotExist);
        };

        let username = username.to_ascii_uppercase();
        let new_password = new_password.to_ascii_uppercase();

        if new_password.len() > MAX_PASS_STR {
            return Err(AccountOpError::PassTooLong);
        }

        let login_db = &LoginDatabase::get();
        let pass_hash = Self::calculate_sha_pass_hash(&username, &new_password);
        LoginDatabase::upd_bnet_password(login_db, params!(pass_hash, account_id)).await?;

        Ok(())
    }

    pub async fn check_password(account_id: u32, password: &str) -> bool {
        let Some(username) = Self::get_name(account_id).await.ok().flatten() else {
            return false;
        };

        let username = username.to_ascii_uppercase();
        let password = password.to_ascii_uppercase();

        let login_db = &LoginDatabase::get();
        let pass_hash = Self::calculate_sha_pass_hash(&username, &password);
        LoginDatabase::sel_bnet_check_password(login_db, params!(account_id, pass_hash))
            .await
            .ok()
            .flatten()
            .is_some()
    }

    pub async fn link_with_game_account(email: &str, game_account_name: &str) -> AccountOpResult<()> {
        let Some(bnetaccount_id) = Self::get_id(email).await? else {
            // account doesn't exist
            return Err(AccountOpError::NameNotExist);
        };

        let Some(game_account_id) = AccountMgr::get_id(game_account_name).await? else {
            return Err(AccountOpError::NameNotExist);
        };
        if Self::get_id_by_game_account(game_account_id).await?.is_some() {
            // got no bnet account to link, bad link
            return Err(AccountOpError::AccountBadLink);
        }

        let login_db = &LoginDatabase::get();

        // TODO: Sounds like a good idea to wrap these 2 queries below in a transaction
        let Some(max_index) = Self::get_max_index(bnetaccount_id).await? else {
            error!(target:"sql::sql", bnetaccount_id=bnetaccount_id, "get max index failed for Battlenet account, this should not happen will be treated as a DB error");
            return Err(AccountOpError::DbInternalError(sqlx::Error::RowNotFound));
        };
        let next_index_to_use = max_index + 1;

        LoginDatabase::upd_bnet_game_account_link(login_db, params!(bnetaccount_id, next_index_to_use, game_account_id)).await?;

        Ok(())
    }

    pub async fn unlink_game_account(game_account_name: &str) -> AccountOpResult<()> {
        let Some(game_account_id) = AccountMgr::get_id(game_account_name).await? else {
            // account doesn't exist
            return Err(AccountOpError::NameNotExist);
        };
        if Self::get_id_by_game_account(game_account_id).await?.is_none() {
            // got no bnet account to unlinked, bad link
            return Err(AccountOpError::AccountBadLink);
        }

        let login_db = &LoginDatabase::get();

        LoginDatabase::upd_bnet_game_account_link(login_db, params!(None::<u32>, None::<u8>, game_account_id)).await?;
        Ok(())
    }

    pub async fn get_id(username: &str) -> AccountOpResult<Option<u32>> {
        let id = LoginDatabase::sel_bnet_account_id_by_email::<_, DbId>(&LoginDatabase::get(), params!(username)).await?;

        Ok(id.map(|v| v.id))
    }

    pub async fn get_name(account_id: u32) -> AccountOpResult<Option<String>> {
        let name = LoginDatabase::sel_bnet_account_email_by_id::<_, DbEmail>(&LoginDatabase::get(), params!(account_id)).await?;
        Ok(name.map(|v| v.email))
    }

    pub async fn get_id_by_game_account(game_account_id: u32) -> AccountOpResult<Option<u32>> {
        let id = LoginDatabase::sel_bnet_account_id_by_game_account::<_, DbBattlenetAccount>(&LoginDatabase::get(), params!(game_account_id)).await?;
        Ok(id.map(|v| v.battlenet_account))
    }

    pub async fn get_max_index(account_id: u32) -> AccountOpResult<Option<u8>> {
        let max_index = LoginDatabase::sel_bnet_max_account_index::<_, DbBnetMaxIndex>(&LoginDatabase::get(), params!(account_id)).await?;
        Ok(max_index.map(|v| v.bnet_max_index))
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
