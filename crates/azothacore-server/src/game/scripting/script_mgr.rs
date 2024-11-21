use std::{collections::BTreeMap, marker::PhantomData};

use azothacore_common::{configuration::DatabaseType, deref_boilerplate};
use bevy::{
    ecs::{system::SystemParam, world::Command},
    prelude::*,
};
use flagset::FlagSet;

use crate::game::{
    globals::object_mgr::DBScriptNameStore,
    scripting::{
        script_defines::{
            account_script::{AccountScript, AccountScriptObject},
            database_script::{DatabaseScript, DatabaseScriptObject},
            world_script::{WorldScript, WorldScriptObject},
        },
        script_object::{AfterLoadScriptObject, IntoScriptObject, Script, ScriptObject, ScriptObjectTrait},
    },
};

// pub trait CommandScript: ScriptObject {
//     /// Should return a pointer to a valid command table (ChatCommand array) to be used by ChatHandler.
//     fn commands(&self) -> Vec<ChatCommand>;
// }

#[derive(SystemParam)]
pub struct ScriptMgr<'w, 's> {
    world:    ScriptRegistry<'w, 's, WorldScriptObject>,
    database: ScriptRegistry<'w, 's, DatabaseScriptObject>,
    account:  ScriptRegistry<'w, 's, AccountScriptObject>,
    // command:  ScriptRegistry<dyn CommandScript>,
}

/// WorldScript functions
impl ScriptMgr<'_, '_> {
    pub fn register_world_script<S>(commands: &mut Commands, script_sys: S)
    where
        S: WorldScript + IntoScriptObject<S, WorldScriptObject> + Send + Sync + 'static,
    {
        commands.add(AddScript::new(script_sys))
    }

    pub fn on_load_custom_database_table(&self, commands: &mut Commands) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_load_custom_database_table {
                commands.run_system(i)
            }
        }
    }

    pub fn on_open_state_change(&self, commands: &mut Commands, open: bool) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_open_state_change {
                commands.run_system_with_input(i, open)
            }
        }
    }

    pub fn on_before_config_load(&self, commands: &mut Commands, reload: bool) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_before_config_load {
                commands.run_system_with_input(i, reload)
            }
        }
    }

    pub fn on_after_config_load(&self, commands: &mut Commands, reload: bool) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_after_config_load {
                commands.run_system_with_input(i, reload)
            }
        }
    }

    pub fn on_before_finalize_player_world_session(&self, commands: &mut Commands, cache_version: u32) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_before_finalize_player_world_session {
                commands.run_system_with_input(i, cache_version)
            }
        }
    }

    pub fn on_motd_change(&self, commands: &mut Commands, new_motd: &str) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_motd_change {
                commands.run_system_with_input(i, new_motd.to_string())
            }
        }
    }

    pub fn on_shutdown_initiate(&self, commands: &mut Commands, shutdown_exit_code: u32, shutdown_mask: u64) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_shutdown_initiate {
                commands.run_system_with_input(i, (shutdown_exit_code, shutdown_mask))
            }
        }
    }

    pub fn on_shutdown_cancel(&self, commands: &mut Commands) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_shutdown_cancel {
                commands.run_system(i)
            }
        }
    }

    pub fn on_update(&self, commands: &mut Commands) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_update {
                commands.run_system(i)
            }
        }
    }

    pub fn on_startup(&self, commands: &mut Commands) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_startup {
                commands.run_system(i)
            }
        }
    }

    pub fn on_shutdown(&self, commands: &mut Commands) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_shutdown {
                commands.run_system(i)
            }
        }
    }

    pub fn on_before_world_initialized(&self, commands: &mut Commands) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_before_world_initialized {
                commands.run_system(i)
            }
        }
    }

    pub fn on_after_unload_all_maps(&self, commands: &mut Commands) {
        for (_, script) in &self.world.script_pointer_list {
            if let Some(i) = script.on_after_unload_all_maps {
                commands.run_system(i)
            }
        }
    }
}

/// DatabaseScript functions
impl ScriptMgr<'_, '_> {
    pub fn register_database_script<S>(commands: &mut Commands, script_sys: S)
    where
        S: DatabaseScript + IntoScriptObject<S, DatabaseScriptObject> + Send + Sync + 'static,
    {
        commands.add(AddScript::new(script_sys))
    }

    pub fn on_after_databases_loaded(&self, commands: &mut Commands, update_flags: FlagSet<DatabaseType>) {
        for (_, script) in &self.database.script_pointer_list {
            if let Some(i) = script.on_after_databases_loaded {
                commands.run_system_with_input(i, update_flags)
            }
        }
    }
}

/// AccountScript functions
impl ScriptMgr<'_, '_> {
    pub fn register_account_script<S>(commands: &mut Commands, script_sys: S)
    where
        S: AccountScript + IntoScriptObject<S, AccountScriptObject> + Send + Sync + 'static,
    {
        commands.add(AddScript::new(script_sys));
    }

    pub fn on_account_login(&self, commands: &mut Commands, account_id: u32) {
        for (_, script) in &self.account.script_pointer_list {
            if let Some(i) = script.on_account_login {
                commands.run_system_with_input(i, account_id)
            }
        }
    }

    // TODO: Impl this azerothcore hook (Not in TC)
    pub fn on_last_ip_update(&self, commands: &mut Commands, account_id: u32, ip: String) {
        for (_, script) in &self.account.script_pointer_list {
            if let Some(i) = script.on_last_ip_update {
                commands.run_system_with_input(i, (account_id, ip.clone()))
            }
        }
    }

    pub fn on_failed_account_login(&self, commands: &mut Commands, account_id: u32) {
        for (_, script) in &self.account.script_pointer_list {
            if let Some(i) = script.on_failed_account_login {
                commands.run_system_with_input(i, account_id)
            }
        }
    }

    pub fn on_email_change(&self, commands: &mut Commands, account_id: u32) {
        for (_, script) in &self.account.script_pointer_list {
            if let Some(i) = script.on_email_change {
                commands.run_system_with_input(i, account_id)
            }
        }
    }

    pub fn on_failed_email_change(&self, commands: &mut Commands, account_id: u32) {
        for (_, script) in &self.account.script_pointer_list {
            if let Some(i) = script.on_failed_email_change {
                commands.run_system_with_input(i, account_id)
            }
        }
    }

    pub fn on_password_change(&self, commands: &mut Commands, account_id: u32) {
        for (_, script) in &self.account.script_pointer_list {
            if let Some(i) = script.on_password_change {
                commands.run_system_with_input(i, account_id)
            }
        }
    }

    pub fn on_failed_password_change(&self, commands: &mut Commands, account_id: u32) {
        for (_, script) in &self.account.script_pointer_list {
            if let Some(i) = script.on_failed_password_change {
                commands.run_system_with_input(i, account_id)
            }
        }
    }

    // TODO: Impl this azerothcore hook (Not in TC)
    pub fn can_account_create_character(&self, commands: &mut Commands, account_id: u32, char_race: u8, char_class: u8) {
        for (_, script) in &self.account.script_pointer_list {
            if let Some(i) = script.can_account_create_character {
                commands.run_system_with_input(i, (account_id, char_race, char_class))
            }
        }
    }
}

// /// CommandScript functions
// impl ScriptMgr {
//     pub fn register_command_script(&mut self, script: Arc<dyn CommandScript>) {
//         self.command.add_script(script);
//     }

//     pub fn chat_commands(&self) -> BTreeMap<String, ChatCommand> {
//         let mut commands = BTreeMap::new();
//         for (_, script) in &self.command.script_pointer_list {
//             for c in script.commands() {
//                 commands.entry(c.name.clone()).or_insert(c);
//             }
//         }
//         commands
//     }
// }

// template class AC_GAME_API ScriptRegistry<AchievementCriteriaScript>;
// template class AC_GAME_API ScriptRegistry<AchievementScript>;
// template class AC_GAME_API ScriptRegistry<AllCreatureScript>;
// template class AC_GAME_API ScriptRegistry<AllGameObjectScript>;
// template class AC_GAME_API ScriptRegistry<AllItemScript>;
// template class AC_GAME_API ScriptRegistry<AllMapScript>;
// template class AC_GAME_API ScriptRegistry<AreaTriggerScript>;
// template class AC_GAME_API ScriptRegistry<ArenaScript>;
// template class AC_GAME_API ScriptRegistry<ArenaTeamScript>;
// template class AC_GAME_API ScriptRegistry<AuctionHouseScript>;
// template class AC_GAME_API ScriptRegistry<BGScript>;
// template class AC_GAME_API ScriptRegistry<BattlegroundMapScript>;
// template class AC_GAME_API ScriptRegistry<BattlegroundScript>;
// template class AC_GAME_API ScriptRegistry<CommandSC>;
// template class AC_GAME_API ScriptRegistry<ConditionScript>;
// template class AC_GAME_API ScriptRegistry<CreatureScript>;
// template class AC_GAME_API ScriptRegistry<DynamicObjectScript>;
// template class AC_GAME_API ScriptRegistry<ElunaScript>;
// template class AC_GAME_API ScriptRegistry<FormulaScript>;
// template class AC_GAME_API ScriptRegistry<GameEventScript>;
// template class AC_GAME_API ScriptRegistry<GameObjectScript>;
// template class AC_GAME_API ScriptRegistry<GlobalScript>;
// template class AC_GAME_API ScriptRegistry<GroupScript>;
// template class AC_GAME_API ScriptRegistry<GuildScript>;
// template class AC_GAME_API ScriptRegistry<InstanceMapScript>;
// template class AC_GAME_API ScriptRegistry<ItemScript>;
// template class AC_GAME_API ScriptRegistry<LootScript>;
// template class AC_GAME_API ScriptRegistry<MailScript>;
// template class AC_GAME_API ScriptRegistry<MiscScript>;
// template class AC_GAME_API ScriptRegistry<MovementHandlerScript>;
// template class AC_GAME_API ScriptRegistry<OutdoorPvPScript>;
// template class AC_GAME_API ScriptRegistry<PetScript>;
// template class AC_GAME_API ScriptRegistry<PlayerScript>;
// template class AC_GAME_API ScriptRegistry<ServerScript>;
// template class AC_GAME_API ScriptRegistry<SpellSC>;
// template class AC_GAME_API ScriptRegistry<SpellScriptLoader>;
// template class AC_GAME_API ScriptRegistry<TransportScript>;
// template class AC_GAME_API ScriptRegistry<UnitScript>;
// template class AC_GAME_API ScriptRegistry<VehicleScript>;
// template class AC_GAME_API ScriptRegistry<WeatherScript>;
// template class AC_GAME_API ScriptRegistry<WorldMapScript>;
// template class AC_GAME_API ScriptRegistry<WorldObjectScript>;

#[derive(SystemParam)]
pub struct ScriptRegistry<'w, 's, Sc>
where
    Sc: Component,
{
    /// The actual list of scripts. This will be accessed concurrently, so it must not be modified
    /// after server startup.
    script_pointer_list: Query<'w, 's, (Entity, &'static Sc)>,
}

/// A mapping of script names to their EntityID in bevy
#[derive(Resource, Default)]
struct RegisteredScriptMapping(BTreeMap<String, Entity>);

deref_boilerplate!(RegisteredScriptMapping, BTreeMap<String, Entity>, 0);

struct AddScript<S, O>(S, PhantomData<O>);

impl<S, O> AddScript<S, O> {
    fn new(script_sys: S) -> Self {
        Self(script_sys, PhantomData)
    }
}

impl<S, O> Command for AddScript<S, O>
where
    S: Script + IntoScriptObject<S, O> + Send + Sync + 'static,
    O: ScriptObjectTrait,
{
    fn apply(self, bevy_world: &mut World) {
        let Self(script_sys, _) = self;
        let base = ScriptObject::create_from_systems(bevy_world, &script_sys);
        let obj = S::create_from_systems(bevy_world, &script_sys);
        if bevy_world.run_system(base.is_afterload_script).unwrap() {
            bevy_world.spawn((base, AfterLoadScriptObject(Some(obj))));
            return;
        }
        if let Err(e) = bevy_world.run_system(base.check_validity).unwrap() {
            warn!(cause=%e, script_name=base.name, "error when checking the validity of the script. Not adding script");
            base.remove_systems_from_bevy(bevy_world);
            obj.remove_systems_from_bevy(bevy_world);
            return;
        }
        register_script_mappings(bevy_world, obj, ScriptBase::New(base));
    }
}

enum ScriptBase {
    New(ScriptObject),
    Existing(String, Entity),
}

fn register_script_mappings<O>(bevy_world: &mut World, obj: O, base: ScriptBase)
where
    O: ScriptObjectTrait,
{
    let (script_name, e) = match base {
        ScriptBase::New(base) => (base.name.clone(), bevy_world.spawn((base, obj)).id()),
        ScriptBase::Existing(s, e) => {
            bevy_world.entity_mut(e).insert(obj);
            (s, e)
        },
    };
    let mut script_mappings = bevy_world.get_resource_or_insert_with(|| RegisteredScriptMapping::default());
    if let Some(old_e) = script_mappings.insert(script_name, e) {
        bevy_world.despawn(old_e);
    }
}

/// Adds the database-bound (i.e. after load) scripts to script management
/// Equivalent to AddALScripts in AC
#[derive(Default)]
struct AddAfterLoadScripts<O>(PhantomData<O>);

impl<O> Command for AddAfterLoadScripts<O>
where
    O: ScriptObjectTrait,
{
    fn apply(self, bevy_world: &mut World) {
        let al_script_entities = bevy_world
            .query_filtered::<Entity, (With<ScriptObject>, With<AfterLoadScriptObject<O>>)>()
            .iter(bevy_world)
            .collect::<Vec<_>>();
        for e in al_script_entities {
            let base = bevy_world.get::<ScriptObject>(e).cloned().unwrap();
            let Some(script) = ({
                let mut ec = bevy_world.entity_mut(e);
                let s = ec.get_mut::<AfterLoadScriptObject<O>>().and_then(|mut s| s.0.take());
                ec.remove::<AfterLoadScriptObject<O>>();
                s
            }) else {
                base.remove_systems_from_bevy(bevy_world);
                bevy_world.despawn(e);
                continue;
            };

            if let Err(err) = bevy_world.run_system(base.check_validity).unwrap() {
                warn!(cause=%err, script_name=base.name, "error when checking the validity of the afterload script. Not adding script");
                base.remove_systems_from_bevy(bevy_world);
                script.remove_systems_from_bevy(bevy_world);
                bevy_world.despawn(e);
                continue;
            }
            if !bevy_world.run_system(base.is_database_bound).unwrap() {
                // We're dealing with a code-only script; just add it.
                register_script_mappings(bevy_world, script, ScriptBase::Existing(base.name, e));
                continue;
            }
            if !bevy_world.resource::<DBScriptNameStore>().contains(&base.name) {
                error!(target:"sql::sql","Script named '{}' is not assigned in the database, not adding", base.name);
                base.remove_systems_from_bevy(bevy_world);
                script.remove_systems_from_bevy(bevy_world);
                bevy_world.despawn(e);
                continue;
            }
            register_script_mappings(bevy_world, script, ScriptBase::Existing(base.name, e));
        }
    }
}
