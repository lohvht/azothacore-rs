use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

use azothacore_common::{
    bevy_app::{az_startup_succeeded, AzStartupFailedEvent, TokioRuntime},
    collision::management::vmap_mgr2::VmapDisabledChecker,
};
use azothacore_database::database_env::WorldDatabase;
use bevy::{
    app::{App, Startup},
    ecs::system::SystemId,
    prelude::{Commands, EventWriter, IntoSystemConfigs, Res, Resource, SystemSet},
};
use flagset::flags;
use num::FromPrimitive;
use num_derive::FromPrimitive;
use sqlx::{prelude::FromRow, query_as};
use tracing::{error, info};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DisableMgrInitialLoadSet;

#[derive(Resource)]
struct LoadDisableMgrCallback(SystemId);

pub fn disable_mgr_plugin(app: &mut App) {
    let load_system_id = app.register_system(DisableMgr::load);
    app.insert_resource(LoadDisableMgrCallback(load_system_id));
    app.add_systems(
        Startup,
        DisableMgr::load_initial.run_if(az_startup_succeeded()).in_set(DisableMgrInitialLoadSet),
    );
}

#[derive(FromPrimitive)]
pub enum DisableType {
    Spell = 0,
    Quest = 1,
    Map = 2,
    Battleground = 3,
    Criteria = 4,
    Outdoorpvp = 5,
    Vmap = 6,
    Mmap = 7,
    LfgMap = 8,
}

flags! {
    pub enum SpellDisable: u8 {
        Player          = 0x1,
        Creature        = 0x2,
        Pet             = 0x4,
        DeprecatedSpell = 0x8,
        Map             = 0x10,
        Area            = 0x20,
        Los             = 0x40,
    }
}

flags! {
    pub enum MMapDisable: u8 {
        Pathfinding = 0x0,
    }
}

struct DisableData {
    flags:  u8,
    param0: BTreeSet<u32>,
    param1: BTreeSet<u32>,
}

#[derive(Resource)]
pub struct DisableMgr {
    disable_map: BTreeMap<DisableType, DisableData>,
}

impl DisableMgr {
    fn load_initial(mut commands: Commands, load_callback: Res<LoadDisableMgrCallback>) {
        commands.run_system(load_callback.0);
    }

    /// LoadDisables in TC / AC
    fn load(mut commands: Commands, rt: Res<TokioRuntime>, world_database: Res<WorldDatabase>, mut ev_startup_failed: EventWriter<AzStartupFailedEvent>) {
        let old_ms_time = Instant::now();
        #[derive(FromRow)]
        struct Disable {
            #[sqlx(rename = "sourceType")]
            source_type: u32,
            entry:       u32,
            flags:       u8,
            params_0:    String,
            params_1:    String,
        }

        let res = match rt.block_on(query_as::<_, Disable>("SELECT sourceType, entry, flags, params_0, params_1 FROM disables").fetch_all(&**world_database)) {
            Err(e) => {
                error!(cause=?e, "error retrieving disables. if terminating if startup");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) if v.is_empty() => {
                info!(target:"server.loading", ">> Loaded 0 disables. DB table `disables` is empty!");
                commands.insert_resource(Self {
                    disable_map: BTreeMap::default(),
                });
                return;
            },
            Ok(v) => v,
        };

        let mut total_count = 0;
        for Disable {
            source_type,
            entry,
            flags,
            params_0,
            params_1,
        } in res
        {
            let Some(typ) = DisableType::from_u32(source_type) else {
                error!(target:"sql.sql", "Invalid type {source_type} specified in `disables` table, skipped.");
                continue;
            };
            todo!("IMPL DISABLE MGR");
            // match typ {
            //     DisableType::Spell => {},
            // }
        }
    }

    fn is_disabled_for(&self, typ: DisableType, entry: u32, unit: Option<()>, flags: u8) -> bool {
        todo!("IMPL IS DISABLED FOR")
    }
}

impl VmapDisabledChecker for DisableMgr {
    fn is_vmap_disabled_for(&self, entry: u32, flags: u8) -> bool {
        self.is_disabled_for(DisableType::Vmap, entry, None, flags)
    }
}
