use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use azothacore_common::{
    az_error,
    bevy_app::{az_startup_succeeded, TokioRuntime},
    configuration::ConfigMgr,
    hex_str,
    utils::net_resolve,
    AccountTypes,
    AzError,
    Locale,
};
use azothacore_database::{
    database_env::{LoginDatabase, LoginPreparedStmts},
    params,
    DbDriver,
};
use bevy::prelude::{App, Commands, IntoSystemConfigs, Real, Res, ResMut, Resource, Startup, SystemSet, Time, Timer, TimerMode, Update};
use flagset::FlagSet;
use futures::StreamExt;
use ipnet::IpNet;
use rand::{rngs::OsRng, RngCore};
use sqlx::Pool;
use tracing::{error, info};

use crate::shared::{
    networking::socket::AddressOrName,
    realms::{BnetRealmHandle, Realm, RealmFlags, RealmType},
};

#[derive(serde::Serialize, serde::Deserialize)]
struct ClientVersion {
    #[serde(rename = "versionMajor")]
    version_major:    u32,
    #[serde(rename = "versionMinor")]
    version_minor:    u32,
    #[serde(rename = "versionRevision")]
    version_revision: u32,
    #[serde(rename = "versionBuild")]
    version_build:    u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RealmEntry {
    #[serde(rename = "wowRealmAddress")]
    wow_realm_address: u32,
    #[serde(rename = "cfgTimezonesID")]
    cfg_timezones_id:  u32,
    #[serde(rename = "populationState")]
    population_state:  u32,
    #[serde(rename = "cfgCategoriesID")]
    cfg_categories_id: u32,
    #[serde(rename = "version")]
    version:           ClientVersion,
    #[serde(rename = "cfgRealmsID")]
    cfg_realms_id:     u32,
    flags:             u32,
    name:              String,
    #[serde(rename = "cfgConfigsID")]
    cfg_configs_id:    u32,
    #[serde(rename = "cfgLanguagesID")]
    cfg_languages_id:  u32,
}

impl RealmEntry {
    fn new(realm: &Realm) -> Self {
        Self {
            wow_realm_address: realm.id.get_address(),
            cfg_timezones_id:  1,
            population_state:  if realm.flag.contains(RealmFlags::Offline) {
                0
            } else {
                (realm.population_level.round() as u32).max(1)
            },
            cfg_categories_id: realm.timezone.into(),
            version:           if let Some(build_info) = get_build_info(realm.build) {
                ClientVersion {
                    version_major:    build_info.major_version,
                    version_minor:    build_info.minor_version,
                    version_revision: build_info.bugfix_version,
                    version_build:    build_info.build,
                }
            } else {
                ClientVersion {
                    version_major:    6,
                    version_minor:    2,
                    version_revision: 4,
                    version_build:    realm.build,
                }
            },
            cfg_realms_id:     realm.id.realm,
            flags:             realm.flag.bits().into(),
            name:              realm.name.clone(),
            cfg_configs_id:    realm.config_id(),
            cfg_languages_id:  1,
        }
    }
}

#[derive(serde::Serialize)]
pub struct RealmState {
    pub update:   Option<RealmEntry>,
    pub deleting: bool,
}

#[derive(serde::Serialize)]
pub struct RealmListUpdates {
    pub updates: Vec<RealmState>,
}

#[derive(serde::Serialize)]
pub struct RealmIPAddress {
    ip:   String,
    port: u32,
}

#[derive(serde::Serialize)]
pub struct RealmIPAddressFamily {
    family:    u32,
    addresses: Vec<RealmIPAddress>,
}

#[derive(serde::Serialize)]
pub struct RealmListServerIPAddresses {
    families: Vec<RealmIPAddressFamily>,
}

struct RealmBuildInfo {
    build:          u32,
    major_version:  u32,
    minor_version:  u32,
    bugfix_version: u32,
}

// List of client builds for verbose version info in realmlist packet
const CLIENT_BUILDS: &[RealmBuildInfo] = &[
    RealmBuildInfo {
        build:          21355,
        major_version:  6,
        minor_version:  2,
        bugfix_version: 4,
    },
    RealmBuildInfo {
        build:          20726,
        major_version:  6,
        minor_version:  2,
        bugfix_version: 3,
    },
    RealmBuildInfo {
        build:          20574,
        major_version:  6,
        minor_version:  2,
        bugfix_version: 2,
    },
    RealmBuildInfo {
        build:          20490,
        major_version:  6,
        minor_version:  2,
        bugfix_version: 2,
    },
    RealmBuildInfo {
        build:          15595,
        major_version:  4,
        minor_version:  3,
        bugfix_version: 4,
    },
    RealmBuildInfo {
        build:          14545,
        major_version:  4,
        minor_version:  2,
        bugfix_version: 2,
    },
    RealmBuildInfo {
        build:          13623,
        major_version:  4,
        minor_version:  0,
        bugfix_version: 6,
    },
    RealmBuildInfo {
        build:          13930,
        major_version:  3,
        minor_version:  3,
        bugfix_version: 5,
    }, // 3.3.5a China Mainland build
    RealmBuildInfo {
        build:          12340,
        major_version:  3,
        minor_version:  3,
        bugfix_version: 5,
    },
    RealmBuildInfo {
        build:          11723,
        major_version:  3,
        minor_version:  3,
        bugfix_version: 3,
    },
    RealmBuildInfo {
        build:          11403,
        major_version:  3,
        minor_version:  3,
        bugfix_version: 2,
    },
    RealmBuildInfo {
        build:          11159,
        major_version:  3,
        minor_version:  3,
        bugfix_version: 0,
    },
    RealmBuildInfo {
        build:          10505,
        major_version:  3,
        minor_version:  2,
        bugfix_version: 2,
    },
    RealmBuildInfo {
        build:          9947,
        major_version:  3,
        minor_version:  1,
        bugfix_version: 3,
    },
    RealmBuildInfo {
        build:          8606,
        major_version:  2,
        minor_version:  4,
        bugfix_version: 3,
    },
    RealmBuildInfo {
        build:          6141,
        major_version:  1,
        minor_version:  12,
        bugfix_version: 3,
    },
    RealmBuildInfo {
        build:          6005,
        major_version:  1,
        minor_version:  12,
        bugfix_version: 2,
    },
    RealmBuildInfo {
        build:          5875,
        major_version:  1,
        minor_version:  12,
        bugfix_version: 1,
    },
];

fn get_build_info(build: u32) -> Option<&'static RealmBuildInfo> {
    CLIENT_BUILDS.iter().find(|c| c.build == build)
}

pub enum JoinRealmError {
    NotPermitted,
    UnknownRealm,
    General,
}

#[derive(sqlx::FromRow)]
pub struct LoginDbRealm {
    id:                     u32,
    name:                   String,
    address:                String,
    #[sqlx(rename = "localAddress")]
    local_address:          String,
    #[sqlx(rename = "localSubnetMask")]
    local_subnet_mask:      String,
    port:                   u16,
    icon:                   u8,
    flag:                   u16,
    timezone:               u8,
    #[sqlx(rename = "allowedSecurityLevel")]
    allowed_security_level: u8,
    population:             f32,
    gamebuild:              u32,
    #[sqlx(rename = "Region")]
    region:                 u8,
    #[sqlx(rename = "Battlegroup")]
    battlegroup:            u8,
}

impl TryFrom<LoginDbRealm> for Realm {
    type Error = AzError;

    fn try_from(value: LoginDbRealm) -> Result<Self, Self::Error> {
        let LoginDbRealm {
            id,
            name,
            address,
            local_address,
            local_subnet_mask,
            port,
            icon,
            flag,
            timezone,
            allowed_security_level,
            population,
            gamebuild,
            region,
            battlegroup,
        } = value;

        let external_address =
            net_resolve((address.as_str(), port)).map_err(|e| az_error!("Could not resolve address {address} for realm \"{name}\", err={e}"))?;
        let local_address = net_resolve((local_address.as_str(), port))
            .map_err(|e| az_error!("Could not resolve localAddress {local_address} for realm \"{name}\", err={e}"))?;

        let local_subnet_mask = net_resolve((local_subnet_mask.as_str(), port))
            .map_err(|e| az_error!("Could not resolve localSubnetMask {local_subnet_mask} for realm \"{name}\", err={e}"))?;

        let mut icon = RealmType::try_from(icon).unwrap_or(RealmType::Normal);
        if matches!(icon, RealmType::FfaPvp) {
            icon = RealmType::Pvp;
        }
        let mut allowed_security_level = AccountTypes::try_from(allowed_security_level).unwrap_or(AccountTypes::SecPlayer);
        if allowed_security_level as u8 > AccountTypes::SecAdministrator as u8 {
            allowed_security_level = AccountTypes::SecAdministrator
        }
        let local_network = IpNet::with_netmask(local_address.ip(), local_subnet_mask.ip())
            .map_err(|e| az_error!("localSubnetMask {local_subnet_mask} for realm \"{name}\" is wrong: err={e}"))?;

        Ok(Self {
            id: BnetRealmHandle::new(region, battlegroup, id),
            build: gamebuild,
            external_address,
            local_address,
            local_network,
            port,
            realm_type: icon,
            name,
            flag: FlagSet::new_truncated(flag),
            timezone,
            allowed_security_level,
            population_level: population,
        })
    }
}

pub trait RealmListConfig: Send + Sync + 'static {
    fn realms_state_update_delay(&self) -> Duration;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RealmListStartSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RealmListUpdateSet;

pub fn realm_list_plugin<C: RealmListConfig>(app: &mut App) {
    app.add_systems(Startup, init_realm_list::<C>.in_set(RealmListStartSet))
        .add_systems(Update, update_realmlists.run_if(az_startup_succeeded()).in_set(RealmListUpdateSet));
}

fn init_realm_list<C: RealmListConfig>(mut commands: Commands, cfg: Res<ConfigMgr<C>>, rt: Res<TokioRuntime>, login_db: Option<Res<LoginDatabase>>) {
    let mut realm_list = RealmList::new(cfg.realms_state_update_delay());
    if let Some(db) = login_db {
        rt.block_on(realm_list.update(db.clone()));
    }
    commands.insert_resource(realm_list);
}

fn update_realmlists(time: Res<Time<Real>>, rt: Res<TokioRuntime>, login_db: Res<LoginDatabase>, mut realm_list: ResMut<RealmList>) {
    realm_list.update_timer.tick(time.delta());
    if !realm_list.has_run_once {
        realm_list.has_run_once = true;
        rt.block_on(realm_list.update(login_db.clone()));
    }
    if !realm_list.update_timer.finished() {
        return;
    }
    rt.block_on(realm_list.update(login_db.clone()));
}

#[derive(Resource)]
pub struct RealmList {
    pub realms:      BTreeMap<BnetRealmHandle, Realm>,
    pub sub_regions: BTreeSet<String>,
    has_run_once:    bool,
    update_timer:    Timer,
}

impl RealmList {
    pub fn new(update_delay: Duration) -> Self {
        Self {
            realms:       BTreeMap::new(),
            sub_regions:  BTreeSet::new(),
            has_run_once: false,
            update_timer: Timer::new(update_delay, TimerMode::Repeating),
        }
    }

    pub fn get_realm_entry_json(&self, id: &BnetRealmHandle, build: u32) -> Option<RealmEntry> {
        let realm = self.realms.get(id)?;
        if realm.flag.contains(RealmFlags::Offline) && realm.build == build {
            return None;
        }
        Some(RealmEntry::new(realm))
    }

    pub fn get_realm_list(&self, build: u32, sub_region: &str) -> RealmListUpdates {
        RealmListUpdates {
            updates: self
                .realms
                .iter()
                .filter_map(|(_, realm)| {
                    if realm.id.get_sub_region_address() != sub_region {
                        return None;
                    }
                    let mut flag = realm.flag;
                    if realm.build != build {
                        flag |= RealmFlags::VersionMismatch;
                    }
                    let mut realm_entry = RealmEntry::new(realm);
                    realm_entry.flags = flag.bits().into();
                    Some(RealmState {
                        deleting: false,
                        update:   Some(realm_entry),
                    })
                })
                .collect(),
        }
    }

    pub fn retrieve_realm_list_server_ip_addresses(
        &self,
        realm_address: u32,
        client_address: &AddressOrName,
        build: u32,
    ) -> Result<RealmListServerIPAddresses, JoinRealmError> {
        let Some(realm) = self.realms.get(&BnetRealmHandle::from_realm_address(realm_address)) else {
            return Err(JoinRealmError::UnknownRealm);
        };
        if realm.flag.contains(RealmFlags::Offline) || realm.build != build {
            return Err(JoinRealmError::NotPermitted);
        }
        let server_addresses = RealmListServerIPAddresses {
            families: vec![RealmIPAddressFamily {
                addresses: vec![RealmIPAddress {
                    ip:   realm.address_for_client(client_address).ip().to_string(),
                    port: realm.port.into(),
                }],
                family:    1,
            }],
        };
        Ok(server_addresses)
    }

    pub async fn join_realm(
        login_db: Pool<DbDriver>,
        client_address: &AddressOrName,
        client_secret: &[u8; 32],
        locale: Locale,
        os: &str,
        account_name: &str,
    ) -> Result<[u8; 32], JoinRealmError> {
        let mut server_secret = [0; 32];
        OsRng.fill_bytes(&mut server_secret);

        let mut key_data = [0; 64];
        key_data[..32].clone_from_slice(client_secret);
        key_data[32..].clone_from_slice(&server_secret);

        if let Err(e) = LoginDatabase::upd_bnet_game_account_login_info(
            &login_db,
            params!(hex_str!(&key_data), client_address.ip_str_or_name(), locale as u8, os, account_name),
        )
        .await
        {
            error!(target:"realmlist", "error trying to login for account {account_name}: err={e}");
            return Err(JoinRealmError::General);
        }
        Ok(server_secret)
    }

    pub async fn update(&mut self, login_db: LoginDatabase) {
        info!(target:"realmlist", "Updating Realm List...");

        let mut existing_realms = BTreeMap::new();
        for p in self.realms.iter() {
            existing_realms.insert(*p.0, p.1.name.clone());
        }
        let mut new_sub_regions = BTreeSet::new();
        let mut new_realms = BTreeMap::new();

        let mut result = LoginDatabase::sel_realmlist::<_, LoginDbRealm>(&*login_db, params!()).await;
        while let Some(res) = result.next().await {
            let realm = match res {
                Err(e) => {
                    error!(target: "realmlist", cause=%e, "DB error when getting realm list, aborting program");
                    break;
                },
                Ok(r) => r,
            };

            let realm: Realm = match realm.try_into() {
                Err(e) => {
                    error!(target:"realmlist", cause=%e, "error converting Realm info from DB entry");
                    continue;
                },
                Ok(r) => r,
            };
            let name = realm.name.as_str();
            if existing_realms.remove(&realm.id).is_some() {
                info!(target:"realmlist", "Updating realm \"{name}\" at {} ({:?}).", realm.external_address, realm.id);
            } else {
                info!(target:"realmlist", "Added realm \"{name}\" at {} ({:?}).", realm.external_address, realm.id);
            }
            new_sub_regions.insert(BnetRealmHandle::new(realm.id.region, realm.id.site, 0).get_address_string());
            new_realms.insert(realm.id, realm);
        }
        for r in existing_realms.values() {
            info!(target:"realmlist", "Removed realm \"{r}\".");
        }
        self.sub_regions = new_sub_regions;
        self.realms = new_realms;
    }
}
