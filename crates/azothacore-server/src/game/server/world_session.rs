use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use azothacore_common::{
    AccountTypes,
    chrono::{DateTime, Utc},
};
use bevy::prelude::*;
use wow_db2::Locale;

use crate::shared::shared_defines::Expansion;

#[derive(Component)]
pub struct WorldSession {
    account_id: u32,
    account_name: String, // i.e. realm_join_ticket
    bnet_account_id: u32,
    security: AccountTypes,
    pub expansion: Expansion,
    pub account_expansion: Expansion,
    mute_time: Option<DateTime<Utc>>,
    os: String,
    session_db_locale_index: Locale,
    recruiter_id: u32,
    is_a_recruiter: bool,
    timeout_time: Duration,

    latency:           AtomicU32,
    client_time_delay: AtomicU32,
}

impl WorldSession {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        account_id: u32,
        name: String,
        bnet_account_id: u32,
        sec: AccountTypes,
        expansion: Expansion,
        account_expansion: Expansion,
        mute_time: Option<DateTime<Utc>>,
        os: String,
        locale: Locale,
        recruiter: u32,
        is_a_recruiter: bool,
        timeout_time: Duration,
    ) -> Self {
        // NOTE: We skip this step from TC / AC
        //       account.online flag is never ever really used anyway in code
        // if (sock)
        // {
        //     m_Address = sock->GetRemoteIpAddress().to_string();
        //     ResetTimeOutTime();
        //     LoginDatabase.PExecute("UPDATE account SET online = 1 WHERE id = %u;", GetAccountId());     // One-time query
        // }
        Self {
            account_id,
            account_name: name,
            bnet_account_id,
            security: sec,
            expansion,
            account_expansion,
            mute_time,
            os,
            session_db_locale_index: locale,
            recruiter_id: recruiter,
            is_a_recruiter,
            timeout_time,
            latency: AtomicU32::new(0),
            client_time_delay: AtomicU32::new(0),
        }
    }

    /// WorldSession::SetLatency and WorldSession::ResetClientTimeDelay in TC / AC
    pub fn set_ping(&self, latency: u32) {
        self.latency.swap(latency, Ordering::SeqCst);
        self.client_time_delay.swap(0, Ordering::SeqCst);
    }

    /// WorldSession::ResetTimeOutTime in TC / AC
    pub fn set_timeout_time(&mut self, duration: Duration) {
        self.timeout_time = duration;
        // TODO: Implement this elsewhere, I'd imagine this function will be moved to sWorld's equivalent
        // in AZC
        // m_timeOutTime = int32(sWorld->getIntConfig(CONFIG_SOCKET_TIMEOUTTIME));
        // if (GetPlayer())
        //     m_timeOutTime = int32(sWorld->getIntConfig(CONFIG_SOCKET_TIMEOUTTIME_ACTIVE));
        // else if (!onlyActive)
        //     m_timeOutTime = int32(sWorld->getIntConfig(CONFIG_SOCKET_TIMEOUTTIME));
    }
}

// NOTE: Copy of TC's constructor commented out for reference
// WorldSession::WorldSession(uint32 id, std::string&& name, uint32 battlenetAccountId, std::shared_ptr<WorldSocket> sock, AccountTypes sec, uint8 expansion, time_t mute_time,
//     std::string os, LocaleConstant locale, uint32 recruiter, bool isARecruiter):
//     m_muteTime(mute_time),
//     m_timeOutTime(0),
//     AntiDOS(this),
//     m_GUIDLow(UI64LIT(0)),
//     _player(NULL),
//     _security(sec),
//     _accountId(id),
//     _accountName(std::move(name)),
//     _battlenetAccountId(battlenetAccountId),
//     m_accountExpansion(expansion),
//     m_expansion(std::min<uint8>(expansion, sWorld->getIntConfig(CONFIG_EXPANSION))),
//     _os(os),
//     _battlenetRequestToken(0),
//     _warden(NULL),
//     _logoutTime(0),
//     m_inQueue(false),
//     m_playerLogout(false),
//     m_playerRecentlyLogout(false),
//     m_playerSave(false),
//     m_sessionDbcLocale(sWorld->GetAvailableDbcLocale(locale)),
//     m_sessionDbLocaleIndex(locale),
//     m_latency(0),
//     m_clientTimeDelay(0),
//     _tutorialsChanged(false),
//     _filterAddonMessages(false),
//     recruiterId(recruiter),
//     isRecruiter(isARecruiter),
//     _RBACData(NULL),
//     expireTime(60000), // 1 min after socket loss, session is deleted
//     forceExit(false),
//     m_currentBankerGUID(),
//     _battlePetMgr(Trinity::make_unique<BattlePetMgr>(this)),
//     _collectionMgr(Trinity::make_unique<CollectionMgr>(this))
// {
//     memset(_tutorials, 0, sizeof(_tutorials));

//     if (sock)
//     {
//         m_Address = sock->GetRemoteIpAddress().to_string();
//         ResetTimeOutTime();
//         LoginDatabase.PExecute("UPDATE account SET online = 1 WHERE id = %u;", GetAccountId());     // One-time query
//     }

//     m_Socket[CONNECTION_TYPE_REALM] = sock;
//     _instanceConnectKey.Raw = UI64LIT(0);
// }
