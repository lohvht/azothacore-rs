use std::collections::{BTreeMap, BTreeSet};

use azothacore_common::{hex_str, AccountTypes};
use azothacore_database::{
    database_env::{CharacterDatabase, CharacterPreparedStmts, LoginDatabase, LoginPreparedStmts},
    params,
    DbAcquire,
    DbExecutor,
};
use sha2::{Digest, Sha256};
use sqlx::{query_as, Connection};
use tokio::{sync::RwLock as AsyncRwLock, time::Instant};
use tracing::{debug, error, info, trace};

use super::{
    rbac::{RawRbacPermId, RbacPermission},
    AccountOpError,
    AccountOpResult,
};
use crate::game::{scripting::script_mgr::SCRIPT_MGR, world::CurrentRealm};

pub const MAX_ACCOUNT_STR: usize = 20;
pub const MAX_PASS_STR: usize = 16;
pub const MAX_EMAIL_STR: usize = 64;

pub struct AccountMgr {
    permissions:         BTreeMap<RawRbacPermId, RbacPermission>,
    default_permissions: BTreeMap<AccountTypes, BTreeSet<RawRbacPermId>>,
}

#[derive(sqlx::FromRow)]
struct RbacPermRow {
    id:   u32,
    name: String,
}

#[derive(sqlx::FromRow)]
struct RbacLinkedPermRow {
    id:                  u32,
    #[sqlx(rename = "linkedId")]
    linkedpermission_id: u32,
}

#[derive(sqlx::FromRow)]
struct RbacDefaultPermRow {
    #[sqlx(rename = "secId")]
    sec_id:        u8,
    #[sqlx(rename = "permissionId")]
    permission_id: u32,
}

impl AccountMgr {
    const fn new() -> Self {
        Self {
            permissions:         BTreeMap::new(),
            default_permissions: BTreeMap::new(),
        }
    }

    pub async fn create_account(username: &str, password: &str, email: &str, bnet_account_id_index: Option<(u32, u8)>) -> AccountOpResult<()> {
        Self::create_account_inner(&LoginDatabase::get(), username, password, email, bnet_account_id_index).await
    }

    pub(super) async fn create_account_inner<'a, A: DbAcquire<'a>>(
        login_db: A,
        username: &str,
        password: &str,
        email: &str,
        bnet_account_id_index: Option<(u32, u8)>,
    ) -> AccountOpResult<()> {
        let mut login_db = login_db.acquire().await?;

        if username.len() > MAX_ACCOUNT_STR {
            // username's too long
            return Err(AccountOpError::NameTooLong);
        }

        if password.len() > MAX_PASS_STR {
            // password's too long
            return Err(AccountOpError::PassTooLong);
        }

        if email.len() > MAX_EMAIL_STR {
            // email too long
            return Err(AccountOpError::EmailTooLong);
        }

        let username = username.to_ascii_uppercase();
        let password = password.to_ascii_uppercase();
        let email = email.to_ascii_uppercase();

        if Self::get_id_inner(&mut *login_db, &username).await?.is_some() {
            // username does already exist
            return Err(AccountOpError::NameAlreadyExist);
        }

        let (bnet_account_id, bnet_index) = bnet_account_id_index.map_or((None, None), |i| (Some(i.0), Some(i.1)));
        let sha_hash = Self::calculate_sha_pass_hash(&username, &password);

        login_db
            .transaction(|txn| {
                Box::pin(async move {
                    LoginDatabase::ins_account(&mut **txn, params!(username, sha_hash, &email, &email, bnet_account_id, bnet_index)).await?;

                    LoginDatabase::ins_realm_characters_init(&mut **txn, params!()).await?;

                    Ok(())
                })
            })
            .await
    }

    // TODO: Implement me: DeleteAccount
    pub async fn delete_account(_account_id: u32) -> AccountOpResult<()> {
        //         let login_db = &LoginDatabase::get();
        //         let char_db = CharacterDatabase::get();
        //         // Check if accounts exists
        //         let exists = LoginDatabase::sel_account_by_id(login_db, params!(account_id)).await?.is_some();
        //         if !exists {
        //             return Err(AccountOpError::NameNotExist);
        //         }

        //         // Obtain accounts characters
        //         let player_guids = CharacterDatabase::sel_chars_by_account_id::<_, (u64,)>(char_db, params!(account_id)).await;

        //         stmt->setUInt32(0, );

        //         result = CharacterDatabase.Query(stmt);

        //         if (result)
        //         {
        //             do
        //             {
        //                 ObjectGuid guid = ObjectGuid::Create<HighGuid::Player>((*result)[0].GetUInt64());

        //                 // Kick if player is online
        //                 if (Player* p = ObjectAccessor::FindConnectedPlayer(guid))
        //                 {
        //                     WorldSession* s = p->GetSession();
        //                     s->KickPlayer();                            // mark session to remove at next session list update
        //                     s->LogoutPlayer(false);                     // logout player without waiting next session list update
        //                 }

        //                 Player::DeleteFromDB(guid, account_id, false);       // no need to update realm characters
        //             } while (result->NextRow());
        //         }

        //         // table realm specific but common for all characters of account for realm
        //         stmt = CharacterDatabase::del_tutorials(char_db, params!());
        //         stmt->setUInt32(0, account_id);
        //         CharacterDatabase.Execute(stmt);

        //         stmt = CharacterDatabase::del_account_data(char_db, params!());
        //         stmt->setUInt32(0, account_id);
        //         CharacterDatabase.Execute(stmt);

        //         stmt = CharacterDatabase::del_character_ban(char_db, params!());
        //         stmt->setUInt32(0, account_id);
        //         CharacterDatabase.Execute(stmt);

        //         SQLTransaction trans = LoginDatabase.BeginTransaction();

        //         stmt = LoginDatabase::del_account(login_db, params!());
        //         stmt->setUInt32(0, account_id);
        //         trans->Append(stmt);

        //         stmt = LoginDatabase::del_account_access(login_db, params!());
        //         stmt->setUInt32(0, account_id);
        //         trans->Append(stmt);

        //         stmt = LoginDatabase::del_realm_characters(login_db, params!());
        //         stmt->setUInt32(0, account_id);
        //         trans->Append(stmt);

        //         stmt = LoginDatabase::del_account_banned(login_db, params!());
        //         stmt->setUInt32(0, account_id);
        //         trans->Append(stmt);

        //         stmt = LoginDatabase::del_account_muted(login_db, params!());
        //         stmt->setUInt32(0, account_id);
        //         trans->Append(stmt);

        //         LoginDatabase.CommitTransaction(trans);
        todo!("IMPLEMENT ME!");
        //         Ok(())
    }

    /// ChangeUsername in TC
    pub async fn change_username_password(account_id: u32, new_username: &str, new_password: &str) -> AccountOpResult<()> {
        Self::change_username_password_inner(&LoginDatabase::get(), account_id, new_username, new_password).await
    }

    pub(super) async fn change_username_password_inner<'a, A: DbAcquire<'a>>(
        login_db: A,
        account_id: u32,
        new_username: &str,
        new_password: &str,
    ) -> AccountOpResult<()> {
        // Check if accounts exists
        let mut login_db = login_db.acquire().await?;
        let result = LoginDatabase::sel_account_by_id(&mut *login_db, params!(account_id)).await?.is_some();

        if !result {
            return Err(AccountOpError::NameNotExist);
        }
        if new_username.len() > MAX_ACCOUNT_STR {
            return Err(AccountOpError::NameTooLong);
        }
        if new_password.len() > MAX_PASS_STR {
            return Err(AccountOpError::PassTooLong);
        }
        let new_username = new_username.to_ascii_uppercase();
        let new_password = new_password.to_ascii_uppercase();

        LoginDatabase::upd_username(
            &mut *login_db,
            params!(&new_username, Self::calculate_sha_pass_hash(&new_username, &new_password), account_id),
        )
        .await?;

        Ok(())
    }

    pub async fn change_password(account_id: u32, new_password: &str) -> AccountOpResult<()> {
        Self::change_password_inner(&LoginDatabase::get(), account_id, new_password).await
    }

    pub(super) async fn change_password_inner<'a, A: DbAcquire<'a>>(login_db: A, account_id: u32, new_password: &str) -> AccountOpResult<()> {
        let mut login_db = login_db.acquire().await?;
        let Some(username) = Self::get_name_inner(&mut *login_db, account_id).await? else {
            SCRIPT_MGR.read().await.on_failed_password_change(account_id);
            return Err(AccountOpError::NameNotExist); // account doesn't exist
        };
        if new_password.len() > MAX_PASS_STR {
            SCRIPT_MGR.read().await.on_failed_password_change(account_id);
            return Err(AccountOpError::PassTooLong);
        }
        let username = username.to_ascii_uppercase();
        let new_password = new_password.to_ascii_uppercase();
        let mut txn = login_db.begin().await?;

        let new_sha_hash = Self::calculate_sha_pass_hash(&username, &new_password);
        LoginDatabase::upd_password(&mut *txn, params!(&new_sha_hash, account_id)).await?;
        LoginDatabase::upd_vs(&mut *txn, params!("", "", username)).await?;
        SCRIPT_MGR.read().await.on_password_change(account_id);
        txn.commit().await?;
        Ok(())
    }

    pub async fn change_email(account_id: u32, new_email: &str) -> AccountOpResult<()> {
        Self::change_email_inner(&LoginDatabase::get(), account_id, new_email).await
    }

    pub(super) async fn change_email_inner<'a, A: DbAcquire<'a>>(login_db: A, account_id: u32, new_email: &str) -> AccountOpResult<()> {
        let mut login_db = login_db.acquire().await?;
        if Self::get_name_inner(&mut *login_db, account_id).await?.is_none() {
            SCRIPT_MGR.read().await.on_failed_email_change(account_id);
            return Err(AccountOpError::NameNotExist); // account doesn't exist
        };
        if new_email.len() > MAX_EMAIL_STR {
            SCRIPT_MGR.read().await.on_failed_email_change(account_id);
            return Err(AccountOpError::EmailTooLong);
        }
        let new_email = new_email.to_ascii_uppercase();

        LoginDatabase::upd_email(&mut *login_db, params!(&new_email, account_id)).await?;
        SCRIPT_MGR.read().await.on_email_change(account_id);
        Ok(())
    }

    pub async fn change_reg_email(account_id: u32, new_email: &str) -> AccountOpResult<()> {
        Self::change_reg_email_inner(&LoginDatabase::get(), account_id, new_email).await
    }

    pub(super) async fn change_reg_email_inner<'a, A: DbAcquire<'a>>(login_db: A, account_id: u32, new_email: &str) -> AccountOpResult<()> {
        let mut login_db = login_db.acquire().await?;
        if Self::get_name_inner(&mut *login_db, account_id).await?.is_none() {
            SCRIPT_MGR.read().await.on_failed_email_change(account_id);
            return Err(AccountOpError::NameNotExist); // account doesn't exist
        };
        if new_email.len() > MAX_EMAIL_STR {
            SCRIPT_MGR.read().await.on_failed_email_change(account_id);
            return Err(AccountOpError::EmailTooLong);
        }
        let new_email = new_email.to_ascii_uppercase();

        LoginDatabase::upd_reg_email(&mut *login_db, params!(&new_email, account_id)).await?;

        SCRIPT_MGR.read().await.on_email_change(account_id);
        Ok(())
    }

    pub async fn check_password(account_id: u32, password: &str) -> bool {
        Self::check_password_inner(&LoginDatabase::get(), account_id, password).await
    }

    pub(super) async fn check_password_inner<'a, A: DbAcquire<'a>>(login_db: A, account_id: u32, password: &str) -> bool {
        let Ok(mut login_db) = login_db.acquire().await else {
            return false;
        };
        let Some(username) = Self::get_name_inner(&mut *login_db, account_id).await.ok().flatten() else {
            return false;
        };
        let username = username.to_ascii_uppercase();
        let password = password.to_ascii_uppercase();

        let pass_hash = Self::calculate_sha_pass_hash(&username, &password);
        LoginDatabase::sel_check_password(&mut *login_db, params!(account_id, pass_hash))
            .await
            .ok()
            .flatten()
            .is_some()
    }

    pub async fn check_email(account_id: u32, new_email: &str) -> bool {
        Self::check_email_inner(&LoginDatabase::get(), account_id, new_email).await
    }

    pub(super) async fn check_email_inner<'e, E: DbExecutor<'e>>(login_db: E, account_id: u32, new_email: &str) -> bool {
        // We simply return false for a non-existing email
        let Some(old_email) = Self::get_email_inner(login_db, account_id).await.ok().flatten() else {
            return false;
        };
        let old_email = old_email.to_ascii_uppercase();
        let new_email = new_email.to_ascii_uppercase();

        old_email == new_email
    }

    pub async fn get_id(username: &str) -> AccountOpResult<Option<u32>> {
        Self::get_id_inner(&LoginDatabase::get(), username).await
    }

    pub(super) async fn get_id_inner<'e, E: DbExecutor<'e>>(login_db: E, username: &str) -> AccountOpResult<Option<u32>> {
        let id = LoginDatabase::get_account_id_by_username::<_, (u32,)>(login_db, params!(username)).await?;
        Ok(id.map(|v| v.0))
    }

    pub async fn get_security(account_id: u32, realm_id: Option<u32>) -> AccountOpResult<AccountTypes> {
        Self::get_security_inner(&LoginDatabase::get(), account_id, realm_id).await
    }

    pub(super) async fn get_security_inner<'e, E: DbExecutor<'e>>(login_db: E, account_id: u32, realm_id: Option<u32>) -> AccountOpResult<AccountTypes> {
        let realm_id_in_db = if let Some(realm_id) = realm_id { i64::from(realm_id) } else { -1 };
        let sec = LoginDatabase::get_gmlevel_by_realmid::<_, (u8,)>(login_db, params!(account_id, realm_id_in_db)).await?;
        Ok(sec.and_then(|sec| sec.0.try_into().ok()).unwrap_or(AccountTypes::SecPlayer))
    }

    pub async fn get_name(account_id: u32) -> AccountOpResult<Option<String>> {
        Self::get_name_inner(&LoginDatabase::get(), account_id).await
    }

    pub(super) async fn get_name_inner<'e, E: DbExecutor<'e>>(login_db: E, account_id: u32) -> AccountOpResult<Option<String>> {
        let name = LoginDatabase::get_username_by_id::<_, (String,)>(login_db, params!(account_id)).await?;
        Ok(name.map(|n| n.0))
    }

    pub async fn get_email(account_id: u32) -> AccountOpResult<Option<String>> {
        Self::get_email_inner(&LoginDatabase::get(), account_id).await
    }

    pub(super) async fn get_email_inner<'e, E: DbExecutor<'e>>(login_db: E, account_id: u32) -> AccountOpResult<Option<String>> {
        let email = LoginDatabase::get_email_by_id::<_, (String,)>(login_db, params!(account_id)).await?;

        Ok(email.map(|n| n.0))
    }

    pub async fn get_characters_count(account_id: u32) -> AccountOpResult<u64> {
        Self::get_characters_count_inner(&CharacterDatabase::get(), account_id).await
    }

    pub(super) async fn get_characters_count_inner<'e, E: DbExecutor<'e>>(char_db: E, account_id: u32) -> AccountOpResult<u64> {
        // check character count
        let counts = CharacterDatabase::sel_sum_chars::<_, (u64,)>(char_db, params!(account_id)).await?;

        Ok(counts.map(|n| n.0).unwrap_or(0))
    }

    pub fn calculate_sha_pass_hash(name: &str, password: &str) -> String {
        let mut sha = Sha256::new();
        sha.update(name.as_bytes());
        sha.update(":");
        sha.update(password.as_bytes());
        let sha_bytes = &sha.finalize()[..];

        hex_str!(sha_bytes)
    }

    pub async fn is_banned_account(name: &str) -> AccountOpResult<bool> {
        Self::is_banned_account_inner(&LoginDatabase::get(), name).await
    }

    pub(super) async fn is_banned_account_inner<'e, E: DbExecutor<'e>>(login_db: E, name: &str) -> AccountOpResult<bool> {
        let account_banned = LoginDatabase::sel_account_banned_by_username::<_, (u32, String)>(login_db, params!(name)).await?;

        let is_not_banned = account_banned.is_empty();
        Ok(!is_not_banned)
    }

    pub async fn update_account_access(account_id: u32, security_level: AccountTypes, realm_id: Option<u32>) -> AccountOpResult<()> {
        Self::update_account_access_inner(&LoginDatabase::get(), account_id, security_level, realm_id).await
    }

    pub(super) async fn update_account_access_inner<'a, A: DbAcquire<'a>>(
        login_db: A,
        account_id: u32,
        security_level: AccountTypes,
        realm_id: Option<u32>,
    ) -> AccountOpResult<()> {
        let mut txn = login_db.begin().await?;
        // Delete old security level from DB,
        if let Some(realm_id) = realm_id {
            LoginDatabase::del_account_access_by_realm(&mut *txn, params!(account_id, realm_id)).await?;
        } else {
            LoginDatabase::del_account_access(&mut *txn, params!(account_id)).await?;
        }
        // also retrieve the realm_id to be saved in DB
        let realm_id_in_db = if let Some(realm_id) = realm_id { i64::from(realm_id) } else { -1 };
        let security_level_in_db = security_level.to_num();
        // Add new security level
        LoginDatabase::ins_account_access(&mut *txn, params!(account_id, security_level_in_db, realm_id_in_db)).await?;
        txn.commit().await?;
        Ok(())
    }

    pub async fn load_rbac(&mut self) -> AccountOpResult<()> {
        self.load_rbac_inner(&LoginDatabase::get(), CurrentRealm::get().id.realm).await
    }

    pub(super) async fn load_rbac_inner<'a, A: DbAcquire<'a>>(&mut self, login_db: A, realm_id: u32) -> AccountOpResult<()> {
        let mut login_db = login_db.acquire().await?;
        self.clear_rbac();

        debug!(target:"rbac", "AccountMgr::LoadRBAC");
        let old_time = Instant::now();

        let mut count1 = 0;
        let mut count2 = 0;
        let mut count3 = 0;

        debug!(target:"rbac", "AccountMgr::LoadRBAC: Loading permissions");
        let result = query_as::<_, RbacPermRow>("SELECT id, name FROM rbac_permissions")
            .fetch_all(&mut *login_db)
            .await?;
        if result.is_empty() {
            info!(target:"server::loading", ">> Loaded 0 account permission definitions. DB table `rbac_permissions` is empty.");
            return Ok(());
        }

        for RbacPermRow { id, name } in result {
            let id = id.try_into();
            self.permissions.entry(id).or_insert(RbacPermission {
                id,
                name,
                linked_permissions: BTreeSet::new(),
            });
            count1 += 1;
        }

        debug!(target:"rbac", "AccountMgr::LoadRBAC: Loading linked permissions");
        let result = query_as::<_, RbacLinkedPermRow>("SELECT id, linkedId FROM rbac_linked_permissions ORDER BY id ASC")
            .fetch_all(&mut *login_db)
            .await?;
        if result.is_empty() {
            info!(target:"server::loading", ">> Loaded 0 linked permissions. DB table `rbac_linked_permissions` is empty.");
            return Ok(());
        }

        for RbacLinkedPermRow { id, linkedpermission_id } in result {
            let permission_id = id.try_into();
            let linkedpermission_id = linkedpermission_id.try_into();
            let Some(permission) = self.permissions.get_mut(&permission_id) else {
                error!(target:"sql.sql", linked_perm_id=?linkedpermission_id, id=?permission_id, "rbac_linked_permissions does not exist in rbac_permissions, Ignored");
                continue;
            };
            if linkedpermission_id == permission_id {
                error!(target:"sql.sql", linked_perm_id=?linkedpermission_id, id=?permission_id, "RBAC Permission has itself as linked permission. Ignored");
                continue;
            }
            permission.linked_permissions.insert(linkedpermission_id);
            count2 += 1;
        }

        debug!(target:"rbac", "AccountMgr::LoadRBAC: Loading default permissions");

        let result = query_as::<_, RbacDefaultPermRow>(
            "SELECT secId, permissionId FROM rbac_default_permissions WHERE (realmId = ? OR realmId = -1) ORDER BY secId ASC",
        )
        .bind(realm_id)
        .fetch_all(&mut *login_db)
        .await?;
        if result.is_empty() {
            info!(target:"server::loading", ">> Loaded 0 default permission definitions. DB table `rbac_default_permissions` is empty.");
            return Ok(());
        }

        for RbacDefaultPermRow { sec_id, permission_id } in result {
            let sec_id = match sec_id.try_into() {
                Err(e) => {
                    error!(target:"sql.sql", sec_id=sec_id, cause=%e, "unexpected sec id. Ignored");
                    continue;
                },
                Ok(i) => i,
            };
            self.default_permissions.entry(sec_id).or_default().insert(permission_id.try_into());
            count3 += 1;
        }

        let speed = old_time.elapsed();
        info!(target:"server::loading", ">> Loaded {count1} permission definitions, {count2} linked permissions and {count3} default permissions in {speed} ms", speed=speed.as_millis());
        Ok(())
    }

    pub fn get_rbac_permission(&self, permission_id: RawRbacPermId) -> Option<&RbacPermission> {
        self.permissions.get(&permission_id)
    }

    fn clear_rbac(&mut self) {
        self.permissions.clear();
        self.default_permissions.clear();
    }

    pub fn get_rbac_default_permissions(&self, sec_level: AccountTypes) -> Option<&BTreeSet<RawRbacPermId>> {
        let def_perms = self.default_permissions.get(&sec_level);
        let def_perm_size = if let Some(p) = def_perms { p.len() } else { 0 };
        trace!(target:"rbac", sec_level=?sec_level, default_perms_size=def_perm_size,  "AccountMgr::GetRBACDefaultPermissions");
        def_perms
    }
}

pub static ACCOUNT_MGR: AsyncRwLock<AccountMgr> = AsyncRwLock::const_new(AccountMgr::new());

#[cfg(test)]
mod tests {

    use azothacore_database::database_env::SHARED_TEST_DB_PERMITS;
    use sqlx::query;

    use super::*;
    use crate::game::accounts::rbac::RbacPermId;

    async fn create_account_for_test<'a, A: DbAcquire<'a>>(login_db: A, user: &str, email: &str, password: &str) -> u32 {
        let mut login_db = login_db.acquire().await.unwrap();

        // Setup a dummy bnet account ID
        LoginDatabase::ins_bnet_account(&mut *login_db, params!(&email, "dummy")).await.unwrap();
        let (bnet_id,) = LoginDatabase::sel_bnet_account_id_by_email(&mut *login_db, params!(email))
            .await
            .ok()
            .flatten()
            .unwrap();

        AccountMgr::create_account_inner(&mut *login_db, user, password, email, Some((bnet_id, 1)))
            .await
            .unwrap();

        // Account ID must exist
        AccountMgr::get_id_inner(&mut *login_db, user).await.ok().flatten().unwrap()
    }

    #[tokio::test]
    async fn it_creates_account() {
        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
        // let _wg = azothacore_common::log::init_console();
        let mut txn = LoginDatabase::get().begin().await.unwrap();

        let username = "name___with_20_chars";
        let email = "sample_email_name_with_64_characters@domain____top.domain_bottom";
        let password = "pwd_with_16_char";

        // Account ID must exist
        let account_id = create_account_for_test(&mut *txn, username, email, password).await;

        let res = query_as::<_, (u32, u32)>("SELECT realmid,numchars from realmcharacters where acctid = ?")
            .bind(account_id)
            .fetch_all(&mut *txn)
            .await
            .unwrap();

        // Has the realm_id 1 (Default) as well as 0.
        assert_eq!(res, vec![(1, 0)]);

        // Check that we cant create with same username
        let res = AccountMgr::create_account_inner(&mut *txn, username, password, email, None).await;
        assert_eq!(res, Err(AccountOpError::NameAlreadyExist), "expect no match but got this: {res:?}");

        // test try several checks here
        assert!(AccountMgr::check_password_inner(&mut *txn, account_id, password).await);
        assert!(AccountMgr::check_email_inner(&mut *txn, account_id, email).await);
        assert!(!AccountMgr::check_password_inner(&mut *txn, account_id, "WRONG_PASS").await);
        assert!(!AccountMgr::check_email_inner(&mut *txn, account_id, "WRONG_EMAIL@WRONG.COM").await);
        let non_existent_account = 9999;
        assert!(!AccountMgr::check_password_inner(&mut *txn, non_existent_account, password).await);
        assert!(!AccountMgr::check_email_inner(&mut *txn, non_existent_account, email).await);
        assert_eq!(
            AccountMgr::get_name_inner(&mut *txn, account_id).await.unwrap(),
            Some(username.to_ascii_uppercase())
        );

        assert_eq!(
            AccountMgr::get_email_inner(&mut *txn, account_id).await.unwrap(),
            Some(email.to_ascii_uppercase())
        );
    }

    #[tokio::test]
    async fn it_cannot_create_account() {
        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
        assert_eq!(
            AccountMgr::create_account(&"a".repeat(MAX_ACCOUNT_STR + 1), "p1", "e1@e1.com", None).await,
            Err(AccountOpError::NameTooLong)
        );
        assert_eq!(
            AccountMgr::create_account("u1", &"a".repeat(MAX_PASS_STR + 1), "e1@e1.com", None).await,
            Err(AccountOpError::PassTooLong)
        );
        assert_eq!(
            AccountMgr::create_account("u1", "p1", &"a".repeat(MAX_EMAIL_STR + 1), None).await,
            Err(AccountOpError::EmailTooLong)
        );
    }

    #[tokio::test]
    async fn it_sets_account_access() {
        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
        let mut txn = LoginDatabase::get().begin().await.unwrap();

        let account_id = create_account_for_test(&mut *txn, "u", "e@e.e", "p").await;
        // Init, account has no security => SEC_PLAYER
        assert_eq!(
            AccountMgr::get_security_inner(&mut *txn, account_id, Some(1)).await.unwrap(),
            AccountTypes::SecPlayer
        );
        assert_eq!(
            AccountMgr::get_security_inner(&mut *txn, account_id, None).await.unwrap(),
            AccountTypes::SecPlayer
        );

        AccountMgr::update_account_access_inner(&mut *txn, account_id, AccountTypes::SecAdministrator, Some(1))
            .await
            .unwrap();
        AccountMgr::update_account_access_inner(&mut *txn, account_id, AccountTypes::SecGamemaster, Some(2))
            .await
            .unwrap();
        assert_eq!(
            AccountMgr::get_security_inner(&mut *txn, account_id, Some(1)).await.unwrap(),
            AccountTypes::SecAdministrator
        );
        assert_eq!(
            AccountMgr::get_security_inner(&mut *txn, account_id, Some(2)).await.unwrap(),
            AccountTypes::SecGamemaster
        );
        assert_eq!(
            AccountMgr::get_security_inner(&mut *txn, account_id, None).await.unwrap(),
            AccountTypes::SecPlayer
        );

        AccountMgr::update_account_access_inner(&mut *txn, account_id, AccountTypes::SecModerator, None)
            .await
            .unwrap();
        assert_eq!(
            AccountMgr::get_security_inner(&mut *txn, account_id, Some(1)).await.unwrap(),
            AccountTypes::SecModerator
        );
        assert_eq!(
            AccountMgr::get_security_inner(&mut *txn, account_id, Some(2)).await.unwrap(),
            AccountTypes::SecModerator
        );
        assert_eq!(
            AccountMgr::get_security_inner(&mut *txn, account_id, None).await.unwrap(),
            AccountTypes::SecModerator
        );
    }

    #[tokio::test]
    async fn it_changes_username_password() {
        // let _wg = azothacore_common::log::init_console();

        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
        let mut txn = LoginDatabase::get().begin().await.unwrap();

        let user1 = "user1";
        let email = "example1@example.domain";
        let password1 = "password1";
        let account_id = create_account_for_test(&mut *txn, user1, email, password1).await;

        let user2 = "user2";
        let password2 = "password2";

        AccountMgr::change_username_password_inner(&mut *txn, account_id, user1, password2)
            .await
            .unwrap();
        assert!(AccountMgr::check_password_inner(&mut *txn, account_id, password2).await);
        assert_eq!(
            AccountMgr::get_name_inner(&mut *txn, account_id).await.unwrap(),
            Some(user1.to_ascii_uppercase())
        );

        // Change username
        AccountMgr::change_username_password_inner(&mut *txn, account_id, user2, password2)
            .await
            .unwrap();
        assert!(AccountMgr::check_password_inner(&mut *txn, account_id, password2).await);
        assert_eq!(
            AccountMgr::get_name_inner(&mut *txn, account_id).await.unwrap(),
            Some(user2.to_ascii_uppercase())
        );

        // Change both
        AccountMgr::change_username_password_inner(&mut *txn, account_id, user1, password1)
            .await
            .unwrap();
        assert!(AccountMgr::check_password_inner(&mut *txn, account_id, password1).await);
        assert_eq!(
            AccountMgr::get_name_inner(&mut *txn, account_id).await.unwrap(),
            Some(user1.to_ascii_uppercase())
        );

        // Pass long
        assert_eq!(
            AccountMgr::change_username_password_inner(&mut *txn, account_id, user1, &"a".repeat(MAX_PASS_STR + 1)).await,
            Err(AccountOpError::PassTooLong)
        );

        // Name long
        assert_eq!(
            AccountMgr::change_username_password_inner(&mut *txn, account_id, &"a".repeat(MAX_ACCOUNT_STR + 1), password1).await,
            Err(AccountOpError::NameTooLong)
        );
    }

    #[tokio::test]
    async fn it_changes_password() {
        // let _wg = azothacore_common::log::init_console();

        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
        let mut txn = LoginDatabase::get().begin().await.unwrap();

        let user1 = "user1";
        let email = "example1@example.domain";
        let password1 = "password1";
        let account_id = create_account_for_test(&mut *txn, user1, email, password1).await;

        let password2 = "password2";

        AccountMgr::change_password_inner(&mut *txn, account_id, password2).await.unwrap();
        assert!(AccountMgr::check_password_inner(&mut *txn, account_id, password2).await);
    }

    #[tokio::test]
    async fn it_does_not_change_password_too_long() {
        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
        let mut txn = LoginDatabase::get().begin().await.unwrap();

        let user1 = "user1";
        let email = "example1@example.domain";
        let password1 = "password1";
        let account_id = create_account_for_test(&mut *txn, user1, email, password1).await;

        // Pass long
        assert_eq!(
            AccountMgr::change_password_inner(&mut *txn, account_id, &"a".repeat(MAX_PASS_STR + 1)).await,
            Err(AccountOpError::PassTooLong)
        );
        assert!(AccountMgr::check_password_inner(&mut *txn, account_id, password1).await);
    }

    #[tokio::test]
    async fn it_does_not_change_password_non_existent_account_id() {
        let non_exist_account_id = 9999;

        assert_eq!(
            AccountMgr::change_password(non_exist_account_id, "123").await,
            Err(AccountOpError::NameNotExist)
        );
    }

    #[tokio::test]
    async fn it_changes_email() {
        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
        let mut txn = LoginDatabase::get().begin().await.unwrap();

        let user1 = "user1";
        let email = "example1@example.domain";
        let password1 = "password1";
        let account_id = create_account_for_test(&mut *txn, user1, email, password1).await;

        let email2 = "example2@example.domain";

        AccountMgr::change_email_inner(&mut *txn, account_id, email2).await.unwrap();
        assert!(AccountMgr::check_email_inner(&mut *txn, account_id, email2).await);
    }

    #[tokio::test]
    async fn it_does_not_change_email_too_long() {
        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
        let mut txn = LoginDatabase::get().begin().await.unwrap();

        let user1 = "user1";
        let email = "example1@example.domain";
        let password1 = "password1";
        let account_id = create_account_for_test(&mut *txn, user1, email, password1).await;

        assert_eq!(
            AccountMgr::change_email_inner(&mut *txn, account_id, &"a".repeat(MAX_EMAIL_STR + 1)).await,
            Err(AccountOpError::EmailTooLong)
        );
        assert!(AccountMgr::check_email_inner(&mut *txn, account_id, email).await);
    }

    #[tokio::test]
    async fn it_does_not_change_email_non_existent_account_id() {
        let non_exist_account_id = 9999;

        assert_eq!(AccountMgr::change_email(non_exist_account_id, "123").await, Err(AccountOpError::NameNotExist));
    }

    async fn check_reg_email<'e, E: DbExecutor<'e>>(conn: E, account_id: u32, email: &str) -> bool {
        query("SELECT 1 FROM account where id = ? AND reg_mail = ? LIMIT 1")
            .bind(account_id)
            .bind(email)
            .fetch_optional(conn)
            .await
            .unwrap()
            .is_some()
    }

    #[tokio::test]
    async fn it_changes_reg_email() {
        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
        let mut txn = LoginDatabase::get().begin().await.unwrap();

        let user1 = "user1";
        let email = "example1@example.domain";
        let password1 = "password1";
        let account_id = create_account_for_test(&mut *txn, user1, email, password1).await;

        let email2 = "example2@example.domain";

        AccountMgr::change_reg_email_inner(&mut *txn, account_id, email2).await.unwrap();
        assert!(check_reg_email(&mut *txn, account_id, email2).await);
    }

    #[tokio::test]
    async fn it_does_not_change_reg_email_too_long() {
        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();
        let mut txn = LoginDatabase::get().begin().await.unwrap();

        let user1 = "user1";
        let email = "example1@example.domain";
        let password1 = "password1";
        let account_id = create_account_for_test(&mut *txn, user1, email, password1).await;

        assert_eq!(
            AccountMgr::change_reg_email_inner(&mut *txn, account_id, &"a".repeat(MAX_EMAIL_STR + 1)).await,
            Err(AccountOpError::EmailTooLong)
        );
        assert!(check_reg_email(&mut *txn, account_id, email).await);
    }

    #[tokio::test]
    async fn it_does_not_change_reg_email_non_existent_account_id() {
        let non_exist_account_id = 9999;

        assert_eq!(
            AccountMgr::change_reg_email(non_exist_account_id, "123").await,
            Err(AccountOpError::NameNotExist)
        );
    }

    #[tokio::test]
    async fn it_checks_banned_accounts() {
        let _p = SHARED_TEST_DB_PERMITS.acquire().await.unwrap();

        let password = "1234";
        let user = "account1";
        let banned_user = "account2";

        assert!(!AccountMgr::is_banned_account(user).await.unwrap());

        let mut txn = LoginDatabase::get().begin().await.unwrap();

        create_account_for_test(&mut *txn, user, "email@email.com", password).await;
        // Make banned account
        let banned_account_id = create_account_for_test(&mut *txn, banned_user, "email2@email.com", password).await;
        LoginDatabase::ins_account_banned(&mut *txn, params!(banned_account_id, 300, "ban_author", "ban_reason"))
            .await
            .unwrap();

        assert!(!AccountMgr::is_banned_account_inner(&mut *txn, user).await.unwrap());
        assert!(AccountMgr::is_banned_account_inner(&mut *txn, banned_user).await.unwrap());
    }

    #[tokio::test]
    async fn it_loads_and_checks_rbac() {
        CurrentRealm::setup_test();
        let mut amgr = ACCOUNT_MGR.write().await;
        amgr.load_rbac().await.unwrap();

        for (sec_level, expected) in [
            (AccountTypes::SecPlayer, Some(&BTreeSet::from_iter([Err(195)]))),
            (AccountTypes::SecModerator, Some(&BTreeSet::from_iter([Err(194)]))),
            (AccountTypes::SecGamemaster, Some(&BTreeSet::from_iter([Err(193)]))),
            (AccountTypes::SecAdministrator, Some(&BTreeSet::from_iter([Err(192)]))),
            (AccountTypes::SecConsole, None),
        ] {
            let got = amgr.get_rbac_default_permissions(sec_level);
            assert_eq!(got, expected);
        }

        for (perm_id, expected) in [
            (
                195.try_into(),
                Some(&RbacPermission {
                    id:                 195.try_into(),
                    name:               "Role: Sec Level Player".to_string(),
                    linked_permissions: BTreeSet::from_iter([
                        Ok(RbacPermId::JoinNormalBg),
                        Ok(RbacPermId::JoinRandomBg),
                        Ok(RbacPermId::JoinArenas),
                        Ok(RbacPermId::JoinDungeonFinder),
                        Ok(RbacPermId::TwoSideCharacterCreation),
                        Ok(RbacPermId::EmailConfirmForPassChange),
                        Err(199),
                    ]),
                }),
            ),
            (
                Ok(RbacPermId::AllowTwoSideTrade),
                Some(&RbacPermission {
                    id:                 Ok(RbacPermId::AllowTwoSideTrade),
                    name:               "Allow trading between factions".to_string(),
                    linked_permissions: BTreeSet::new(),
                }),
            ),
        ] {
            let got = amgr.get_rbac_permission(perm_id);
            assert_eq!(got, expected);
        }
    }
}
