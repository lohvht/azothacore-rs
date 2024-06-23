use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use tracing::error;

#[allow(non_snake_case)]
mod structs;

use bevy::prelude::*;
pub use structs::*;

use crate::{
    bevy_app::{az_startup_succeeded, AzStartupDryRunEvent, AzStartupFailedEvent},
    AzResult,
};

pub trait Config: serde::de::DeserializeOwned + Send + Sync + 'static {
    fn load<P: AsRef<Path>>(config_toml: P) -> AzResult<Self> {
        from_env_toml(config_toml)
    }

    fn reload(&mut self, new: Self) {
        *self = new;
    }
}

#[derive(Resource)]
pub struct ConfigMgr<C> {
    pub filename:   PathBuf,
    pub is_dry_run: bool,
    config:         C,
}

impl<C> Deref for ConfigMgr<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl<C> DerefMut for ConfigMgr<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

#[derive(Event)]
pub struct ConfigReloadEvent;

#[derive(Event)]
pub struct ConfigReloadFinishedEvent<C>(PhantomData<C>);

impl<C> Default for ConfigReloadFinishedEvent<C> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(SystemSet)]
pub enum ConfigMgrSet<C> {
    LoadInitial(PhantomData<C>),
    Reload(PhantomData<C>),
}

impl<C> ConfigMgrSet<C> {
    pub fn load_initial() -> Self {
        Self::LoadInitial(PhantomData::<C>)
    }

    pub fn reload() -> Self {
        Self::Reload(PhantomData::<C>)
    }
}

impl<C> Clone for ConfigMgrSet<C> {
    fn clone(&self) -> Self {
        match self {
            Self::LoadInitial(p) => Self::LoadInitial(*p),
            Self::Reload(p) => Self::Reload(*p),
        }
    }
}

impl<C> PartialEq for ConfigMgrSet<C> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LoadInitial(l0), Self::LoadInitial(r0)) => l0 == r0,
            (Self::Reload(l0), Self::Reload(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl<C> Debug for ConfigMgrSet<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ctype = std::any::type_name::<C>();
        match self {
            Self::LoadInitial(_) => f.debug_tuple("LoadInitial").field(&ctype).finish(),
            Self::Reload(_) => f.debug_tuple("Reload").field(&ctype).finish(),
        }
    }
}

impl<C> Hash for ConfigMgrSet<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<C> Eq for ConfigMgrSet<C> {}

/// config_mgr_plugin is a bevy plugin to register a configuration manager
/// for a given filename for a given configuration type. as well as the
/// management of the contents of the config loaded.
///
/// During the [PreStartup] schedule, the plugin registers the [ConfigMgr]
/// resource which holds the current filename, as well as the config that
/// is loaded (if successful).
///
/// Otherwise, if unsuccessful, an [AzStartupFailedEvent] is emitted and
/// the app should attempt to trigger an AppExit during the [PostStartup]
/// phase
///
/// Users can emit an [ConfigReloadEvent] if they would like to attempt to
/// reload their config file. This is reload will be done during the [FixedUpdate]
/// schedule and a corresponding [ConfigReloadFinishedEvent<C>] will be emitted
/// to show that a reload was attempted successfully
pub fn config_mgr_plugin<C, P>(init_file_name: P, dry_run: bool) -> impl Fn(&mut bevy::prelude::App)
where
    C: Config,
    P: AsRef<Path>,
{
    let filename = init_file_name.as_ref().to_path_buf();
    move |app: &mut App| {
        let f = filename.clone();
        app.add_event::<ConfigReloadEvent>()
            .add_event::<ConfigReloadFinishedEvent<C>>()
            .add_systems(PreStartup, load_initial_configs::<_, C>(f, dry_run).in_set(ConfigMgrSet::<C>::load_initial()))
            .add_systems(
                FixedUpdate,
                reload_config::<C>.run_if(az_startup_succeeded()).in_set(ConfigMgrSet::<C>::reload()),
            );
    }
}

// Loads the main app configuration.
fn load_initial_configs<P, C>(path: P, is_dry_run: bool) -> impl FnMut(Commands, EventWriter<AzStartupDryRunEvent>, EventWriter<AzStartupFailedEvent>)
where
    P: AsRef<Path>,
    C: Config,
{
    move |mut commands: Commands, mut ev_startup_dryrun: EventWriter<AzStartupDryRunEvent>, mut ev_startup_failed: EventWriter<AzStartupFailedEvent>| {
        let cfg: C = match Config::load(&path) {
            Err(e) => {
                ev_startup_failed.send_default();
                error!(cause=%e, "error initialising config");
                return;
            },
            Ok(c) => c,
        };
        commands.insert_resource(ConfigMgr {
            filename: path.as_ref().to_path_buf(),
            config: cfg,
            is_dry_run,
        });

        if is_dry_run {
            ev_startup_dryrun.send_default();
        }
    }
}

fn reload_config<C>(
    mut reload_cfg_events: EventReader<ConfigReloadEvent>,
    mut cfg: ResMut<ConfigMgr<C>>,
    mut ev_cfg_reload_finished: EventWriter<ConfigReloadFinishedEvent<C>>,
) where
    C: Config,
{
    let mut reloaded = false;
    let mut attempted_reload = false;
    for _ev in reload_cfg_events.read() {
        if !attempted_reload {
            // Reloading configuraion should not fail at all, for now we just log
            if let Err(err) = cfg.reload_from_path() {
                warn!(cause=%err, path=%cfg.filename.display(), "error reloading from path, using old configs");
            } else {
                reloaded = true;
            }
            attempted_reload = true;
        }
    }
    if reloaded {
        ev_cfg_reload_finished.send_default();
    }
}

impl<C> ConfigMgr<C>
where
    C: Config,
{
    fn reload_from_path(&mut self) -> AzResult<()> {
        let new = C::load(&self.filename)?;
        self.config.reload(new);
        println!("RELOADING! {}", self.filename.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use figment::Jail;
    use serde::{Deserialize, Serialize};
    use serde_default::DefaultFromSerde;
    use tokio::sync::mpsc::{channel, error::TryRecvError};

    use super::*;
    use crate::bevy_app::{bevy_app, AzStartupState, DEFAULT_FRAME_RATE};

    #[derive(Deserialize, Serialize, DefaultFromSerde, PartialEq)]
    pub struct TestConfig {
        #[serde(default)]
        integer: i32,
    }

    impl Config for TestConfig {}

    #[test]
    fn it_inits_the_initial_cfg_mgr() {
        Jail::expect_with(|jail| {
            let cfg_path = "config.toml";
            jail.create_file(cfg_path, "integer = 3")?;

            let mut app = bevy_app();
            assert!(app.world.get_resource::<ConfigMgr<TestConfig>>().is_none());
            app.add_plugins(config_mgr_plugin::<TestConfig, _>(cfg_path, false));
            app.update();
            assert_eq!(app.world.resource::<ConfigMgr<TestConfig>>().integer, 3);
            Ok(())
        });
    }

    #[test]
    fn it_reloads_cfg_mgr() {
        Jail::expect_with(|jail| {
            let cfg_path = "config.toml";
            let mut app = bevy_app();
            let (snd, mut rcv) = channel(1);
            let closure = move |mut ev_reloaded: EventReader<ConfigReloadFinishedEvent<TestConfig>>| {
                for _ in ev_reloaded.read() {
                    snd.try_send("Reloaded".to_string()).unwrap();
                }
            };
            assert!(app.world.get_resource::<ConfigMgr<TestConfig>>().is_none());
            app.add_plugins(config_mgr_plugin::<TestConfig, _>(cfg_path, false))
                .add_systems(Update, closure);
            app.update();
            assert_eq!(app.world.resource::<ConfigMgr<TestConfig>>().integer, 0);

            jail.create_file(cfg_path, "integer = 3")?;

            app.world.send_event(ConfigReloadEvent);
            // Simulate fixed update
            sleep(Duration::from_secs_f64(2.0 / DEFAULT_FRAME_RATE));
            app.update();
            let res = rcv.try_recv().unwrap();
            assert_eq!(app.world.resource::<ConfigMgr<TestConfig>>().integer, 3);
            assert_eq!(res, "Reloaded");

            Ok(())
        });
    }

    #[test]
    fn app_does_not_run_update_as_cfg_mgr_is_dry_run() {
        let mut app = bevy_app();
        let (snd, mut rcv) = channel(1);
        app.add_plugins(config_mgr_plugin::<TestConfig, _>("", true)).add_systems(
            Update,
            (move || {
                snd.try_send(()).unwrap();
            })
            .run_if(az_startup_succeeded()),
        );

        assert_eq!(*app.world.resource::<State<AzStartupState>>().get(), AzStartupState::Succeeded);
        app.update();
        assert_eq!(*app.world.resource::<State<AzStartupState>>().get(), AzStartupState::DryRun);
        let err = rcv.try_recv();
        assert_eq!(err, Err(TryRecvError::Empty));
    }

    #[test]
    #[should_panic(expected = "Resource requested by")]
    fn cfg_mgr_systemset_reordering_works_panic_if_resource_asked_before_load_initial() {
        #[derive(Component)]
        struct A;

        #[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
        struct ClosureSet;

        fn closure(_cfg: Res<ConfigMgr<TestConfig>>, mut commands: Commands) {
            commands.spawn(A);
        }
        let mut app = bevy_app();

        app.add_plugins(config_mgr_plugin::<TestConfig, _>("", true))
            .add_systems(PreStartup, closure.in_set(ClosureSet))
            .configure_sets(PreStartup, ClosureSet.before(ConfigMgrSet::<TestConfig>::load_initial()));

        app.update();
    }

    #[test]
    fn cfg_mgr_systemset_reordering_works_if_resource_asked_after_load_initial() {
        #[derive(Component)]
        struct A(i32);

        #[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
        struct ClosureSet;

        let (snd, mut rcv) = channel(1);

        let closure = move |_cfg: Res<ConfigMgr<TestConfig>>, mut commands: Commands| {
            let e = commands.spawn(A(2)).id();
            snd.try_send(e).unwrap();
        };
        let mut app = bevy_app();

        app.add_plugins(config_mgr_plugin::<TestConfig, _>("", true))
            .add_systems(PreStartup, closure.in_set(ClosureSet))
            .configure_sets(PreStartup, ConfigMgrSet::<TestConfig>::load_initial().before(ClosureSet));

        app.update();
        let eref = app.world.entity(rcv.try_recv().unwrap()).get::<A>().unwrap().0;
        assert_eq!(eref, 2);
    }
}
