use std::{
    collections::{BTreeMap, BTreeSet},
    net::{self, IpAddr, ToSocketAddrs},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

use azothacore_common::{az_error, get_g, mut_g, AccountTypes, AzResult};
use flagset::FlagSet;
use sqlx::Row;
use tokio::runtime::Handle as TokioRtHandle;
use tracing::{debug, error, info};

use crate::{
    database::{
        database_env::{LoginDatabase, LoginPreparedStmts},
        params,
    },
    shared::realms::{BnetRealmHandle, Realm, RealmType},
};

pub struct RealmList {
    update_interval_duration: Duration,
    realms:                   BTreeMap<BnetRealmHandle, Realm>,
    sub_regions:              BTreeSet<String>,
}

fn net_resolve(addr_str: &str, port: u16) -> AzResult<IpAddr> {
    let addr = addr_str.parse::<net::IpAddr>()?;
    if (addr, port).to_socket_addrs()?.next().is_none() {
        Err(az_error!("Could not resolve address {addr_str}:{port}"))
    } else {
        Ok(addr)
    }
}

impl RealmList {
    pub fn r() -> RwLockReadGuard<'static, Self> {
        get_g!(REALM_LIST)
    }

    fn m() -> RwLockWriteGuard<'static, Self> {
        mut_g!(REALM_LIST)
    }

    pub const fn new() -> Self {
        Self {
            update_interval_duration: Duration::from_secs(10),
            realms:                   BTreeMap::new(),
            sub_regions:              BTreeSet::new(),
        }
    }

    pub fn init(async_handler: &TokioRtHandle, cancel_token: tokio_util::sync::CancellationToken, update_interval_in_seconds: u64) {
        let mut w = Self::m();

        w.update_interval_duration = Duration::from_secs(update_interval_in_seconds);
        // Get the content of the realmlist table in the database
        async_handler.spawn(Self::update_realms(cancel_token));
    }

    async fn update_realms(cancel_token: tokio_util::sync::CancellationToken) {
        let mut interval = tokio::time::interval(Self::r().update_interval_duration);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            let _t = tokio::select! {
                _ = cancel_token.cancelled() => {
                    break;
                }
                i = interval.tick() => i,
            };
            debug!(target:"realmlist", "Updating Realm List...");

            let mut existing_realms = BTreeMap::new();
            for p in Self::r().realms.iter() {
                existing_realms.insert(*p.0, p.1.name.clone());
            }
            let mut new_sub_regions = BTreeSet::new();
            let mut new_realms = BTreeMap::new();

            let result = match LoginDatabase::sel_realmlist(LoginDatabase::get(), params!()).await {
                Err(e) => {
                    error!(target:"realmlist", err=e.to_string(), "error getting the new list of realms from login database");
                    continue;
                },
                Ok(r) => r,
            };

            for fields in result {
                let realm_id = fields.get("id");
                let name = fields.get("name");
                let external_address: String = fields.get("address");
                let local_address: String = fields.get("localAddress");
                let local_subnet_mask: String = fields.get("localSubnetMask");
                let port = fields.get("port");

                let external_address = match net_resolve(&external_address, port) {
                    Err(e) => {
                        error!(target:"realmlist", err=e.to_string(), "Could not resolve address {external_address} for realm \"{}\" id {}", name, realm_id);
                        continue;
                    },
                    Ok(a) => a,
                };
                let local_address = match net_resolve(&local_address, port) {
                    Err(e) => {
                        error!(target:"realmlist", err=e.to_string(), "Could not resolve localAddress {local_address} for realm \"{}\" id {}", name, realm_id);
                        continue;
                    },
                    Ok(a) => a,
                };
                let local_subnet_mask = match net_resolve(&local_subnet_mask, port) {
                    Err(e) => {
                        error!(target:"realmlist", err=e.to_string(),"Could not resolve localSubnetMask {local_subnet_mask} for realm \"{}\" id {}", name, realm_id);
                        continue;
                    },
                    Ok(a) => a,
                };
                let mut icon = RealmType::try_from(fields.get::<u8, _>("icon")).unwrap_or(RealmType::Normal);
                if matches!(icon, RealmType::FfaPvp) {
                    icon = RealmType::Pvp;
                }
                let flag = fields.get("flag");
                let timezone = fields.get("timezone");
                let mut allowed_security_level =
                    AccountTypes::try_from(fields.get::<u8, _>("allowedSecurityLevel")).unwrap_or(AccountTypes::SecPlayer);
                if allowed_security_level as u8 > AccountTypes::SecAdministrator as u8 {
                    allowed_security_level = AccountTypes::SecAdministrator
                }
                let pop = fields.get("population");
                let build = fields.get("gamebuild");
                let region = fields.get("Region");
                let battlegroup = fields.get("Battlegroup");

                let id = BnetRealmHandle::new(region, battlegroup, realm_id);
                let realm = Realm {
                    id,
                    build,
                    name,
                    external_address,
                    local_address,
                    local_subnet_mask,
                    port,
                    realm_type: icon,
                    flag: FlagSet::new_truncated(flag),
                    timezone,
                    allowed_security_level,
                    population_level: pop,
                };
                let name = realm.name.as_str();
                if existing_realms.contains_key(&id) {
                    info!(target:"realmlist", "Added realm \"{name}\" at {external_address}:{port}.");
                } else {
                    debug!(target:"realmlist", "Updating realm \"{name}\" at {external_address}:{port}.");
                }
                new_realms.insert(id, realm);

                new_sub_regions.insert(BnetRealmHandle::new(region, battlegroup, 0).get_address_string());
            }

            for r in existing_realms.values() {
                info!(target:"realmlist", "Removed realm \"{r}\".");
            }
            {
                let mut realm_list_w = Self::m();
                realm_list_w.sub_regions = new_sub_regions;
                realm_list_w.realms = new_realms;
            }
        }
    }
}

static REALM_LIST: RwLock<RealmList> = RwLock::new(RealmList::new());
