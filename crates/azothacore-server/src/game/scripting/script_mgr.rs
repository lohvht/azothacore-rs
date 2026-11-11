use std::{collections::BTreeMap, marker::PhantomData};

use azothacore_common::{configuration::DatabaseType, deref_boilerplate};
use bevy::{
    ecs::system::{Command, SystemId, SystemParam},
    prelude::*,
};
use flagset::FlagSet;
use tracing::{error, warn};
use wow_db2::Locale;

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

/// none → `()`, one → the value, several → a tuple. Matches the existing `In<…>` script inputs.
macro_rules! hook_input {
    () => { () };
    ($a:ident) => { $a };
    ($($a:ident),+) => { ($($a),+) };
}

macro_rules! script_hooks {
    ( $( $obj:ident { $( $(#[$meta:meta])* $kind:ident $name:ident / $queued:ident $args:tt $(-> $out:ty)? ; )* } )* ) => {
        impl ScriptMgr {
            $($( script_hooks!(@$kind [$(#[$meta])*] $obj $name $queued $args $(-> $out)?); )*)*
        }
    };

    (@deferred_then [$(#[$m:meta])*] $name:ident $queued:ident ($($arg:ident : $ty:ty),*) -> $out:ty) => {
        $(#[$m])*
        pub fn $queued(commands: &mut Commands, $($arg: $ty,)* then: impl FnOnce(&mut World, $out) + Send + 'static) {
            commands.queue(move |world: &mut World| {
                let res = Self::$name(world, $($arg),*);
                then(world, res);
            });
        }
    };

    (@notify [$(#[$m:meta])*] $obj:ident $name:ident $queued:ident ($($arg:ident : $ty:ty),*)) => {
        $(#[$m])*
        pub fn $name(world: &mut World, $($arg: $ty),*) {
            notify_hooks(world, stringify!($name), |s: &$obj| s.$name, hook_input!($($arg),*));
        }
        $(#[$m])*
        pub fn $queued(commands: &mut Commands, $($arg: $ty),*) {
            commands.queue(move |world: &mut World| Self::$name(world, $($arg),*));
        }
    };

    (@boolean $m:tt $obj:ident $name:ident $queued:ident $args:tt -> $out:ty) => {
        script_hooks!(@boolean_impl true $m $obj $name $queued $args -> $out);
    };

    (@boolean_default_false $m:tt $obj:ident $name:ident $queued:ident $args:tt -> $out:ty) => {
        script_hooks!(@boolean_impl false $m $obj $name $queued $args -> $out);
    };

    (@boolean_impl $default:literal [$(#[$m:meta])*] $obj:ident $name:ident $queued:ident ($($arg:ident : $ty:ty),*) -> $out:ty) => {
        $(#[$m])*
        pub fn $name(world: &mut World, $($arg: $ty),*) -> $out {
            boolean_hooks(world, stringify!($name), |s: &$obj| s.$name, hook_input!($($arg),*), $default)
        }
        script_hooks!(@deferred_then [$(#[$m])*] $name $queued ($($arg: $ty),*) -> $out);
    };

    // (value: T): script input InMut<T>.
    (@value [$(#[$m:meta])*] $obj:ident $name:ident $queued:ident ($val:ident : $vty:ty)) => {
        $(#[$m])*
        pub fn $name(world: &mut World, $val: $vty) -> $vty {
            value_hooks(world, stringify!($name), |s: &$obj| s.$name, $val)
        }
        script_hooks!(@deferred_then [$(#[$m])*] $name $queued ($val: $vty) -> $vty);
    };

    // (value: T; read-only args): script input (In<args>, InMut<T>).
    (@value [$(#[$m:meta])*] $obj:ident $name:ident $queued:ident ($val:ident : $vty:ty; $($arg:ident : $ty:ty),+)) => {
        $(#[$m])*
        pub fn $name(world: &mut World, $val: $vty, $($arg: $ty),+) -> $vty {
            value_hooks_with_input(world, stringify!($name), |s: &$obj| s.$name, hook_input!($($arg),+), $val)
        }
        script_hooks!(@deferred_then [$(#[$m])*] $name $queued ($val: $vty, $($arg: $ty),+) -> $vty);
    };
}

// pub trait CommandScript: ScriptObject {
//     /// Should return a pointer to a valid command table (ChatCommand array) to be used by ChatHandler.
//     fn commands(&self) -> Vec<ChatCommand>;
// }

pub struct ScriptMgr;

/// Retrieves the System IDs
fn hook_ids<O, I, R>(world: &mut World, hook: impl Fn(&O) -> Option<SystemId<I, R>>) -> Vec<SystemId<I, R>>
where
    O: Component,
    I: SystemInput + 'static,
    R: 'static,
{
    let mut q = world.query::<&O>();
    q.iter(world).filter_map(hook).collect()
}

fn notify_hooks<O, I>(world: &mut World, name: &str, hook: impl Fn(&O) -> Option<SystemId<I>>, input: I::Inner<'static>)
where
    O: Component,
    I: SystemInput + 'static,
    I::Inner<'static>: Clone,
{
    for id in hook_ids(world, hook) {
        if let Err(e) = world.run_system_with(id, input.clone()) {
            error!(target:"scripts", cause=?e, hook=name, "script hook failed to run");
        }
    }
}

/// AC boolean hooks: the first script returning `!default` wins; no scripts (or none disagreeing) → `default`.
/// `default = true` is `CALL_ENABLED_BOOLEAN_HOOKS`, `default = false` is `..._WITH_DEFAULT_FALSE`.
fn boolean_hooks<O, I>(world: &mut World, name: &str, hook: impl Fn(&O) -> Option<SystemId<I, bool>>, input: I::Inner<'static>, default: bool) -> bool
where
    O: Component,
    I: SystemInput + 'static,
    I::Inner<'static>: Clone,
{
    let overridden = hook_ids(world, hook).into_iter().any(|id| {
        world.run_system_with(id, input.clone()).unwrap_or_else(|e| {
            error!(target:"scripts", cause=?e, hook=name, "script hook failed to run");
            default
        }) != default
    });
    default != overridden
}

/// Each script edits `value` in place, as with TC/AC `T&` out-params; the caller gets it back.
fn value_hooks<O, T>(world: &mut World, name: &str, hook: impl Fn(&O) -> Option<SystemId<InMut<'static, T>>>, mut value: T) -> T
where
    O: Component,
    T: 'static,
{
    for id in hook_ids(world, hook) {
        if let Err(e) = world.run_system_with(id, &mut value) {
            error!(target:"scripts", cause=?e, hook=name, "script hook failed to run");
        }
    }
    value
}

/// `value_hooks` for hooks that also pass read-only inputs, given to each script as `In<C>`.
fn value_hooks_with_input<O, C, T>(world: &mut World, name: &str, hook: impl Fn(&O) -> Option<SystemId<(In<C>, InMut<'static, T>)>>, ctx: C, mut value: T) -> T
where
    O: Component,
    C: Clone + 'static,
    T: 'static,
{
    for id in hook_ids(world, hook) {
        if let Err(e) = world.run_system_with(id, (ctx.clone(), &mut value)) {
            error!(target:"scripts", cause=?e, hook=name, "script hook failed to run");
        }
    }
    value
}

impl ScriptMgr {
    pub fn register_world_script<S>(commands: &mut Commands, script_sys: S)
    where
        S: WorldScript + IntoScriptObject<S, WorldScriptObject> + Send + Sync + 'static,
    {
        commands.queue(AddScript::new(script_sys))
    }

    pub fn register_database_script<S>(commands: &mut Commands, script_sys: S)
    where
        S: DatabaseScript + IntoScriptObject<S, DatabaseScriptObject> + Send + Sync + 'static,
    {
        commands.queue(AddScript::new(script_sys))
    }

    pub fn register_account_script<S>(commands: &mut Commands, script_sys: S)
    where
        S: AccountScript + IntoScriptObject<S, AccountScriptObject> + Send + Sync + 'static,
    {
        commands.queue(AddScript::new(script_sys));
    }
}

script_hooks! {
    WorldScriptObject {
        /// ScriptMgr::OnLoadCustomDatabaseTable in AC
        /// Called when loading custom database tables
        notify on_load_custom_database_table / on_load_custom_database_table_deferred ();
        /// ScriptMgr::OnOpenStateChange in TC/AC
        /// Called when the open/closed state of the world changes.
        notify on_open_state_change / on_open_state_change_deferred (open: bool);
        /// ScriptMgr::OnBeforeConfigLoad in AC
        /// Called before the world configuration is (re)loaded.
        /// AC runs this before the config file is read; the deferred variant runs after.
        notify on_before_config_load / on_before_config_load_deferred (reload: bool);
        /// ScriptMgr::OnAfterConfigLoad in AC / ScriptMgr::OnConfigLoad in TC
        /// Called after the world configuration is (re)loaded.
        notify on_after_config_load / on_after_config_load_deferred (reload: bool);
        /// ScriptMgr::OnBeforeFinalizePlayerWorldSession in AC
        ///
        /// @brief This hook runs before finalizing the player world session. Can be also used to mutate the cache version of the Client.
        ///
        /// @param version The cache version that we will be sending to the Client.
        ///
        value  on_before_finalize_player_world_session / on_before_finalize_player_world_session_deferred (cache_version: u32);
        /// ScriptMgr::OnMotdChange in TC/AC
        /// Called before the message of the day is changed.
        value on_motd_change / on_motd_change_deferred (new_motd: String; locale: Locale);
        /// ScriptMgr::OnShutdownInitiate in TC/AC
        /// Called when a world shutdown is initiated.
        notify on_shutdown_initiate / on_shutdown_initiate_deferred (shutdown_exit_code: u32, shutdown_mask: u64);
        /// ScriptMgr::OnShutdownCancel in TC/AC
        /// Called when a world shutdown is cancelled.
        notify on_shutdown_cancel / on_shutdown_cancel_deferred ();
        /// ScriptMgr::OnWorldUpdate in TC/AC
        /// Called on every world tick (don't execute too heavy code here).
        notify on_update / on_update_deferred (diff: u32);
        /// ScriptMgr::OnStartup in TC/AC
        /// Called when the world is started.
        notify on_startup / on_startup_deferred ();
        /// ScriptMgr::OnShutdown in TC/AC
        /// Called when the world is actually shut down.
        notify on_shutdown / on_shutdown_deferred ();
        /// ScriptMgr::OnBeforeWorldInitialized in AC
        ///
        /// @brief This hook runs after all scripts loading and before itialized
        ///
        notify on_before_world_initialized / on_before_world_initialized_deferred ();
        /// ScriptMgr::OnAfterUnloadAllMaps in AC
        /// Called after all maps are unloaded from core
        notify on_after_unload_all_maps / on_after_unload_all_maps_deferred ();
    }
    DatabaseScriptObject {
        /// ScriptMgr::OnAfterDatabasesLoaded in AC
        /// Called after all databases are loaded, with the update flags from the loader
        notify on_after_databases_loaded / on_after_databases_loaded_deferred (update_flags: FlagSet<DatabaseType>);
    }
    AccountScriptObject {
        /// ScriptMgr::OnAccountLogin in TC/AC
        /// Called when an account logged in successfully
        notify on_account_login / on_account_login_deferred (account_id: u32);
        /// ScriptMgr::OnLastIpUpdate in AC
        /// Called when an ip logged in successfully
        notify on_last_ip_update / on_last_ip_update_deferred (account_id: u32, ip: String);
        /// ScriptMgr::OnFailedAccountLogin in TC/AC
        /// Called when an account login failed
        notify on_failed_account_login / on_failed_account_login_deferred (account_id: u32);
        /// ScriptMgr::OnEmailChange in TC/AC
        /// Called when Email is successfully changed for Account
        notify on_email_change / on_email_change_deferred (account_id: u32);
        /// ScriptMgr::OnFailedEmailChange in TC/AC
        /// Called when Email failed to change for Account
        notify on_failed_email_change / on_failed_email_change_deferred (account_id: u32);
        /// ScriptMgr::OnPasswordChange in TC/AC
        /// Called when Password is successfully changed for Account
        notify on_password_change / on_password_change_deferred (account_id: u32);
        /// ScriptMgr::OnFailedPasswordChange in TC/AC
        /// Called when Password failed to change for Account
        notify on_failed_password_change / on_failed_password_change_deferred (account_id: u32);
        /// ScriptMgr::CanAccountCreateCharacter in AC
        /// Called when creating a character on the Account
        boolean can_account_create_character / can_account_create_character_deferred (account_id: u32, char_race: u8, char_class: u8) -> bool;
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
    type Out = ();

    fn apply(self, bevy_world: &mut World) {
        let Self(script_sys, _) = self;
        let base = ScriptObject::create_from_systems(bevy_world, &script_sys);
        let obj = S::create_from_systems(bevy_world, &script_sys);
        if bevy_world.run_system(base.is_afterload_script).unwrap() {
            bevy_world.spawn((base, AfterLoadScriptObject(Some(obj))));
            return;
        }
        if let Err(e) = bevy_world.run_system(base.check_validity).unwrap() {
            warn!(cause=?e, script_name=base.name, "error when checking the validity of the script. Not adding script");
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
    let mut script_mappings = bevy_world.get_resource_or_insert_with(RegisteredScriptMapping::default);

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
    type Out = ();

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
                warn!(cause=?err, script_name=base.name, "error when checking the validity of the afterload script. Not adding script");
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

#[cfg(test)]
mod tests {
    // TODO: Cleanup the tests below, primarily the ones that use the following directly:
    // - notify_hooks
    // - boolean_hooks
    // - value_hooks
    // - value_hooks_with_input
    //
    // These were a rough draft provided by an LLM that looks okay at first glance

    use super::*;

    fn world_with(register: impl FnOnce(&mut Commands)) -> World {
        let mut world = World::new();
        register(&mut world.commands());
        world.flush();
        world
    }

    // --- test scripts ---

    struct TestBooleanHookAccountScript {
        name:  &'static str,
        allow: bool,
    }
    impl Script for TestBooleanHookAccountScript {
        fn name(&self) -> String {
            self.name.into()
        }
    }
    impl AccountScript for TestBooleanHookAccountScript {
        fn can_account_create_character(&self) -> Option<impl System<In = In<(u32, u8, u8)>, Out = bool>> {
            let allow = self.allow;
            Some(IntoSystem::into_system(move |_: In<(u32, u8, u8)>| allow))
        }
    }

    /// Implements no hook at all; must be skipped, not treated as a vote.
    struct TestNoHookBooleanAccountScript;
    impl Script for TestNoHookBooleanAccountScript {}
    impl AccountScript for TestNoHookBooleanAccountScript {}

    struct TestMutateAndReturnInputWorldScript(&'static str);
    impl Script for TestMutateAndReturnInputWorldScript {
        fn name(&self) -> String {
            self.0.into()
        }
    }
    impl WorldScript for TestMutateAndReturnInputWorldScript {
        fn on_before_finalize_player_world_session(&self) -> Option<impl System<In = InMut<'static, u32>, Out = ()>> {
            Some(IntoSystem::into_system(|InMut(v): InMut<u32>| *v += 1))
        }
    }

    #[derive(Resource, Default)]
    struct Seen(Vec<(u32, String)>);

    struct TestRecordIpAccountScript(&'static str);
    impl Script for TestRecordIpAccountScript {
        fn name(&self) -> String {
            self.0.into()
        }
    }
    impl AccountScript for TestRecordIpAccountScript {
        fn on_last_ip_update(&self) -> Option<impl System<In = In<(u32, String)>, Out = ()>> {
            Some(IntoSystem::into_system(|In(v): In<(u32, String)>, mut seen: ResMut<Seen>| seen.0.push(v)))
        }
    }

    // --- boolean ---

    #[test]
    fn it_allows_character_creation_when_no_script_votes() {
        let mut world = world_with(|c| ScriptMgr::register_account_script(c, TestNoHookBooleanAccountScript));
        assert!(ScriptMgr::can_account_create_character(&mut world, 1, 2, 3));
    }

    #[test]
    fn it_rejects_character_creation_when_any_script_says_no() {
        let mut world = world_with(|c| {
            ScriptMgr::register_account_script(c, TestBooleanHookAccountScript { name: "yes", allow: true });
            ScriptMgr::register_account_script(c, TestBooleanHookAccountScript { name: "no", allow: false });
        });
        assert!(!ScriptMgr::can_account_create_character(&mut world, 1, 2, 3));
    }

    #[test]
    fn it_needs_one_true_script_when_default_is_false() {
        let hook = |s: &AccountScriptObject| s.can_account_create_character;
        let mut world = World::new();
        assert!(!boolean_hooks(&mut world, "t", hook, (1, 2, 3), false));

        let mut world = world_with(|c| ScriptMgr::register_account_script(c, TestBooleanHookAccountScript { name: "no", allow: false }));
        assert!(!boolean_hooks(&mut world, "t", hook, (1, 2, 3), false));

        let mut world = world_with(|c| {
            ScriptMgr::register_account_script(c, TestBooleanHookAccountScript { name: "no", allow: false });
            ScriptMgr::register_account_script(c, TestBooleanHookAccountScript { name: "yes", allow: true });
        });
        assert!(boolean_hooks(&mut world, "t", hook, (1, 2, 3), false));
    }

    // --- value ---

    #[test]
    fn it_lets_every_script_edit_the_value() {
        let mut world = world_with(|c| {
            ScriptMgr::register_world_script(c, TestMutateAndReturnInputWorldScript("a"));
            ScriptMgr::register_world_script(c, TestMutateAndReturnInputWorldScript("b"));
        });
        assert_eq!(ScriptMgr::on_before_finalize_player_world_session(&mut world, 5), 7);
    }

    #[test]
    fn it_keeps_the_value_when_a_script_fails_to_run() {
        let mut world = world_with(|c| {
            ScriptMgr::register_world_script(c, TestMutateAndReturnInputWorldScript("a"));
            ScriptMgr::register_world_script(c, TestMutateAndReturnInputWorldScript("b"));
            ScriptMgr::register_world_script(c, TestMutateAndReturnInputWorldScript("stale"));
        });
        let e = world.resource::<RegisteredScriptMapping>()["stale"];
        let id = world.get::<WorldScriptObject>(e).unwrap().on_before_finalize_player_world_session.unwrap();
        world.unregister_system(id).unwrap();

        assert_eq!(ScriptMgr::on_before_finalize_player_world_session(&mut world, 5), 7);
    }

    #[test]
    fn it_passes_read_only_input_next_to_the_value() {
        struct TagMotd;
        impl Script for TagMotd {}
        impl WorldScript for TagMotd {
            fn on_motd_change(&self) -> Option<impl System<In = (In<Locale>, InMut<'static, String>), Out = ()>> {
                Some(IntoSystem::into_system(|(In(l), InMut(m)): (In<Locale>, InMut<String>)| {
                    if l == Locale::deDE {
                        m.push_str(" (de)");
                    }
                }))
            }
        }
        let mut world = world_with(|c| ScriptMgr::register_world_script(c, TagMotd));
        assert_eq!(ScriptMgr::on_motd_change(&mut world, "hi".into(), Locale::deDE), "hi (de)");
        assert_eq!(ScriptMgr::on_motd_change(&mut world, "hi".into(), Locale::enUS), "hi");
    }

    // --- notify, several args ---

    #[test]
    fn it_gives_each_script_its_own_copy_of_the_input() {
        let mut world = world_with(|c| {
            ScriptMgr::register_account_script(c, TestRecordIpAccountScript("a"));
            ScriptMgr::register_account_script(c, TestRecordIpAccountScript("b"));
        });
        world.init_resource::<Seen>();
        ScriptMgr::on_last_ip_update(&mut world, 7, "1.2.3.4".into());
        assert_eq!(world.resource::<Seen>().0, vec![(7, "1.2.3.4".into()); 2]);
    }

    // --- deferred ---

    #[test]
    fn it_runs_deferred_hooks_only_at_flush() {
        let mut world = world_with(|c| ScriptMgr::register_account_script(c, TestRecordIpAccountScript("a")));
        world.init_resource::<Seen>();

        ScriptMgr::on_last_ip_update_deferred(&mut world.commands(), 7, "1.2.3.4".into());
        assert!(world.resource::<Seen>().0.is_empty());
        world.flush();
        assert_eq!(world.resource::<Seen>().0.len(), 1);
    }

    #[test]
    fn it_hands_the_deferred_result_to_then() {
        #[derive(Resource)]
        struct Answer(bool);

        let mut world = world_with(|c| ScriptMgr::register_account_script(c, TestBooleanHookAccountScript { name: "no", allow: false }));
        ScriptMgr::can_account_create_character_deferred(&mut world.commands(), 1, 2, 3, |w, ok| w.insert_resource(Answer(ok)));
        world.flush();
        assert!(!world.resource::<Answer>().0);
    }

    /// Guards the `worldserver/src/main.rs` reorder: resources queued before the hook exist when it runs.
    #[test]
    fn it_sees_resources_queued_before_the_deferred_hook() {
        #[derive(Resource)]
        struct FakeDb;
        #[derive(Resource, Default)]
        struct SawDb(bool);

        struct CheckDb;
        impl Script for CheckDb {}
        impl DatabaseScript for CheckDb {
            fn on_after_databases_loaded(&self) -> Option<impl System<In = In<FlagSet<DatabaseType>>, Out = ()>> {
                Some(IntoSystem::into_system(
                    |_: In<FlagSet<DatabaseType>>, db: Option<Res<FakeDb>>, mut saw: ResMut<SawDb>| saw.0 = db.is_some(),
                ))
            }
        }
        let mut world = world_with(|c| ScriptMgr::register_database_script(c, CheckDb));
        world.init_resource::<SawDb>();

        let mut commands = world.commands();
        commands.insert_resource(FakeDb);
        ScriptMgr::on_after_databases_loaded_deferred(&mut commands, FlagSet::default());
        world.flush();
        assert!(world.resource::<SawDb>().0);
    }
}
