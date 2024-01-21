use std::collections::{BTreeMap, BTreeSet};

use azothacore_common::{hex_str, AccountTypes};
use azothacore_database::{
    database_env::{CharacterDatabase, CharacterPreparedStmts, LoginDatabase, LoginPreparedStmts},
    params,
};
use sha2::{Digest, Sha256};
use sqlx::{query_as, Connection};
use tokio::{sync::RwLock as AsyncRwLock, time::Instant};
use tracing::{debug, error, info, trace};

use super::{
    db_internal,
    rbac::{RawRbacPermId, RbacPermission},
    rbac_err_internal,
    AccountOpError,
    AccountOpResult,
    DbId,
};
use crate::game::{accounts::rbac::RbacData, world::CurrentRealm};

pub const MAX_ACCOUNT_STR: usize = 20;
pub const MAX_PASS_STR: usize = 16;

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
        if username.len() > MAX_ACCOUNT_STR {
            // username's too long
            return Err(AccountOpError::NameTooLong);
        }

        if password.len() > MAX_PASS_STR {
            // password's too long
            return Err(AccountOpError::PassTooLong);
        }

        let username = username.to_ascii_uppercase();
        let password = password.to_ascii_uppercase();
        let email = email.to_ascii_uppercase();

        if Self::get_id(&email).await?.is_some() {
            // username does already exist
            return Err(AccountOpError::NameAlreadyExist);
        }

        let login_db = LoginDatabase::get();
        let (bnet_account_id, bnet_index) = bnet_account_id_index.map_or((None, None), |i| (Some(i.0), Some(i.1)));
        let sha_hash = Self::calculate_sha_pass_hash(&username, &password);

        login_db
            .acquire()
            .await
            .map_err(db_internal("unable to retrive connection for transaction"))?
            .transaction(|txn| {
                Box::pin(async move {
                    LoginDatabase::ins_account(&mut **txn, params!(username, sha_hash, &email, &email, bnet_account_id, bnet_index)).await?;

                    LoginDatabase::ins_realm_characters_init(&mut **txn, params!()).await?;

                    Ok(())
                })
            })
            .await
            .map_err(db_internal("error inserting account / realm characters init in txn"))?;
        // everything's fine
        Ok(())
    }

    // pub async fn delete_account(account_id: u32) -> AccountOpResult<()>
    //     {
    //         let login_db = LoginDatabase::get();
    //         let char_db = CharacterDatabase::get();
    //         // Check if accounts exists
    //         let exists = LoginDatabase::sel_account_by_id(login_db, params!(account_id)).await.map_err(db_internal("delete account name check failed"))?.is_some();
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

    //         Ok(())
    //     }

    //     static AccountOpResult ChangeUsername(account_id: u32, std::string newUsername, std::string newPassword)
    //     {
    //         // Check if accounts exists
    //         PreparedStatement* stmt = LoginDatabase::sel_account_by_id(login_db, params!());
    //         stmt->setUInt32(0, account_id);
    //         PreparedQueryResult result = LoginDatabase.Query(stmt);

    //         if (!result)
    //             return Err(AccountOpError::NameNotExist);

    //         if (utf8length(newUsername) > MAX_ACCOUNT_STR)
    //             return Err(AccountOpError::NameTooLong);

    //         if (utf8length(newPassword) > MAX_PASS_STR)
    //             return Err(AccountOpError::PassTooLong);

    //         Utf8ToUpperOnlyLatin(newUsername);
    //         Utf8ToUpperOnlyLatin(newPassword);

    //         stmt = LoginDatabase::upd_username(login_db, params!());

    //         stmt->setString(0, newUsername);
    //         stmt->setString(1, Self::calculate_sha_pass_hash(newUsername, newPassword));
    //         stmt->setUInt32(2, account_id);

    //         LoginDatabase.Execute(stmt);

    //         Ok(())
    //     }
    //     static AccountOpResult ChangePassword(account_id: u32, std::string newPassword)
    //     {
    //         std::string username;

    //         if (!GetName(account_id, username))
    //         {
    //             sScriptMgr->OnFailedPasswordChange(account_id);
    //             return Err(AccountOpError::NameNotExist);                          // account doesn't exist
    //         }

    //         if (utf8length(newPassword) > MAX_PASS_STR)
    //         {
    //             sScriptMgr->OnFailedPasswordChange(account_id);
    //             return Err(AccountOpError::PassTooLong);
    //         }

    //         Utf8ToUpperOnlyLatin(username);
    //         Utf8ToUpperOnlyLatin(newPassword);

    //         PreparedStatement* stmt = LoginDatabase::upd_password(login_db, params!());

    //         stmt->setString(0, Self::calculate_sha_pass_hash(username, newPassword));
    //         stmt->setUInt32(1, account_id);

    //         LoginDatabase.Execute(stmt);

    //         stmt = LoginDatabase::upd_vs(login_db, params!());

    //         stmt->setString(0, "");
    //         stmt->setString(1, "");
    //         stmt->setString(2, username);

    //         LoginDatabase.Execute(stmt);

    //         sScriptMgr->OnPasswordChange(account_id);
    //         Ok(())
    //     }

    //     static AccountOpResult ChangeEmail(account_id: u32, std::string newEmail)
    //     {
    //         std::string username;

    //         if (!GetName(account_id, username))
    //         {
    //             sScriptMgr->OnFailedEmailChange(account_id);
    //             return Err(AccountOpError::NameNotExist);                          // account doesn't exist
    //         }

    //         if (utf8length(newEmail) > MAX_EMAIL_STR)
    //         {
    //             sScriptMgr->OnFailedEmailChange(account_id);
    //             return Err(AccountOpError::EmailTooLong);
    //         }

    //         Utf8ToUpperOnlyLatin(username);
    //         Utf8ToUpperOnlyLatin(newEmail);

    //         PreparedStatement* stmt = LoginDatabase::upd_email(login_db, params!());

    //         stmt->setString(0, newEmail);
    //         stmt->setUInt32(1, account_id);

    //         LoginDatabase.Execute(stmt);

    //         sScriptMgr->OnEmailChange(account_id);
    //         Ok(())
    //     }
    //     static AccountOpResult ChangeRegEmail(account_id: u32, std::string newEmail)
    //     {
    //         std::string username;

    //         if (!GetName(account_id, username))
    //         {
    //             sScriptMgr->OnFailedEmailChange(account_id);
    //             return Err(AccountOpError::NameNotExist);                          // account doesn't exist
    //         }

    //         if (utf8length(newEmail) > MAX_EMAIL_STR)
    //         {
    //             sScriptMgr->OnFailedEmailChange(account_id);
    //             return Err(AccountOpError::EmailTooLong);
    //         }

    //         Utf8ToUpperOnlyLatin(username);
    //         Utf8ToUpperOnlyLatin(newEmail);

    //         PreparedStatement* stmt = LoginDatabase::upd_reg_email(login_db, params!());

    //         stmt->setString(0, newEmail);
    //         stmt->setUInt32(1, account_id);

    //         LoginDatabase.Execute(stmt);

    //         sScriptMgr->OnEmailChange(account_id);
    //         Ok(())
    //     }
    //     static bool CheckPassword(account_id: u32, std::string password)
    //     {
    //         std::string username;

    //         if (!GetName(account_id, username))
    //             return false;

    //         Utf8ToUpperOnlyLatin(username);
    //         Utf8ToUpperOnlyLatin(password);

    //         PreparedStatement* stmt = LoginDatabase::sel_check_password(login_db, params!());
    //         stmt->setUInt32(0, account_id);
    //         stmt->setString(1, Self::calculate_sha_pass_hash(username, password));
    //         PreparedQueryResult result = LoginDatabase.Query(stmt);

    //         return (result) ? true : false;
    //     }
    //     static bool CheckEmail(account_id: u32, std::string newEmail)
    //     {
    //         std::string oldEmail;

    //         // We simply return false for a non-existing email
    //         if (!GetEmail(account_id, oldEmail))
    //             return false;

    //         Utf8ToUpperOnlyLatin(oldEmail);
    //         Utf8ToUpperOnlyLatin(newEmail);

    //         if (strcmp(oldEmail.c_str(), newEmail.c_str()) == 0)
    //             return true;

    //         return false;
    //     }

    pub async fn get_id(username: &str) -> AccountOpResult<Option<u32>> {
        let id = LoginDatabase::get_account_id_by_username::<_, DbId>(LoginDatabase::get(), params!(username))
            .await
            .map_err(db_internal("error when getting account ID from DB"))?;
        Ok(id.map(|v| v.id))
    }

    pub async fn get_security(account_id: u32) -> AccountOpResult<AccountTypes> {
        let sec = LoginDatabase::get_account_access_gmlevel::<_, (u8,)>(LoginDatabase::get(), params!(account_id))
            .await
            .map_err(db_internal("error when getting account security level from DB"))?;
        Ok(sec.and_then(|sec| sec.0.try_into().ok()).unwrap_or(AccountTypes::SecPlayer))
    }

    pub async fn get_security_by_realm_id(account_id: u32, realm_id: Option<u32>) -> AccountOpResult<AccountTypes> {
        let realm_id_in_db = if let Some(realm_id) = realm_id { i64::from(realm_id) } else { -1 };
        let sec = LoginDatabase::get_gmlevel_by_realmid::<_, (u8,)>(LoginDatabase::get(), params!(account_id, realm_id_in_db))
            .await
            .map_err(db_internal("error when getting account security level by realm_id from DB"))?;
        Ok(sec.and_then(|sec| sec.0.try_into().ok()).unwrap_or(AccountTypes::SecPlayer))
    }

    pub async fn get_name(account_id: u32) -> AccountOpResult<Option<String>> {
        let name = LoginDatabase::get_username_by_id::<_, (String,)>(LoginDatabase::get(), params!(account_id))
            .await
            .map_err(db_internal("error when getting account username from DB"))?;

        Ok(name.map(|n| n.0))
    }

    pub async fn get_email(account_id: u32) -> AccountOpResult<Option<String>> {
        let email = LoginDatabase::get_email_by_id::<_, (String,)>(LoginDatabase::get(), params!(account_id))
            .await
            .map_err(db_internal("error when getting account email from DB"))?;

        Ok(email.map(|n| n.0))
    }

    pub async fn get_characters_count(account_id: u32) -> AccountOpResult<u64> {
        // check character count
        let counts = CharacterDatabase::sel_sum_chars::<_, (u64,)>(CharacterDatabase::get(), params!(account_id))
            .await
            .map_err(db_internal("error when getting char counts from DB"))?;

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
        let is_not_banned = Self::list_banned_account_by_name(name).await?.is_empty();
        Ok(!is_not_banned)
    }

    pub async fn list_banned_account_by_name(name: &str) -> AccountOpResult<Vec<(u32, String)>> {
        LoginDatabase::sel_account_banned_by_username(LoginDatabase::get(), params!(name))
            .await
            .map_err(db_internal("error listing banned accounts by name"))
    }

    pub async fn list_banned_account_all() -> AccountOpResult<Vec<(u32, String)>> {
        LoginDatabase::sel_account_banned_all(LoginDatabase::get(), params!())
            .await
            .map_err(db_internal("error listing all banned accounts"))
    }

    pub async fn update_account_access(rbac: &mut RbacData, security_level: AccountTypes, realm_id: Option<u32>) -> AccountOpResult<()> {
        let account_id = rbac.id;
        rbac.set_security_level(security_level)
            .await
            .map_err(rbac_err_internal("set RBAC sec level error"))?;
        LoginDatabase::get()
            .acquire()
            .await
            .map_err(db_internal("unable to retrive connection for transaction to update account access"))?
            .transaction(|txn| {
                Box::pin(async move {
                    // Delete old security level from DB,
                    if let Some(realm_id) = realm_id {
                        LoginDatabase::del_account_access_by_realm(&mut **txn, params!(account_id, realm_id)).await?;
                    } else {
                        LoginDatabase::del_account_access(&mut **txn, params!(account_id)).await?;
                    }
                    // also retrieve the realm_id to be saved in DB
                    let realm_id_in_db = if let Some(realm_id) = realm_id { i64::from(realm_id) } else { -1 };
                    let security_level_in_db = security_level.to_num();
                    // Add new security level
                    LoginDatabase::ins_account_access(&mut **txn, params!(account_id, security_level_in_db, realm_id_in_db)).await?;

                    Ok(())
                })
            })
            .await
            .map_err(db_internal("error updating account access in txn"))
    }

    pub async fn load_rbac(&mut self) -> AccountOpResult<()> {
        self.clear_rbac();

        debug!(target:"rbac", "AccountMgr::LoadRBAC");
        let old_time = Instant::now();

        let mut count1 = 0;
        let mut count2 = 0;
        let mut count3 = 0;

        debug!(target:"rbac", "AccountMgr::LoadRBAC: Loading permissions");
        let login_db = LoginDatabase::get();
        let result = query_as::<_, RbacPermRow>("SELECT id, name FROM rbac_permissions")
            .fetch_all(login_db)
            .await
            .map_err(db_internal("error loading rbac perms"))?;
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
            .fetch_all(login_db)
            .await
            .map_err(db_internal("error loading rbac_linked_permissions"))?;
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
        .bind(CurrentRealm::get().id.realm)
        .fetch_all(login_db)
        .await
        .map_err(db_internal("error loading rbac_default_permissions"))?;
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
