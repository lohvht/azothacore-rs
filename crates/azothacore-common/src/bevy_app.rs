use bevy::{
    asset::AssetPlugin,
    diagnostic::DiagnosticsPlugin,
    prelude::{App, AppExit, Event, EventReader, EventWriter, Fixed, HierarchyPlugin, NextState, PostStartup, Res, ResMut, Resource, Time, TransformPlugin},
    state::{
        app::{AppExtStates, StatesPlugin},
        condition::in_state,
        state::{State, States},
    },
    MinimalPlugins,
};
use tokio::runtime::Runtime;
use tracing::{error, info};

use crate::deref_boilerplate;

/// Default we use for FixedUpdates in azothacore, can be overwritten again
pub const DEFAULT_FRAME_RATE: f64 = 144.0; // Bevy default 64;

/// The current server state.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AzStartupState {
    #[default]
    Succeeded,
    DryRun,
    Failed,
}

pub fn az_startup_succeeded() -> impl FnMut(Option<Res<State<AzStartupState>>>) -> bool + Clone {
    in_state(AzStartupState::Succeeded)
}

/// A newtype that implements bevy resource that wraps around a tokio runtime
#[derive(Resource)]
pub struct TokioRuntime(pub Runtime);

deref_boilerplate!(TokioRuntime, Runtime, 0);

/// See [bevy_app] for the documentation regarding how this event should be used
#[derive(Event, Default)]
pub struct AzStartupFailedEvent;

/// See [bevy_app] for the documentation regarding how this event should be used
#[derive(Event, Default)]
pub struct AzStartupDryRunEvent;

/// shared bevy app set up used by all cores.
/// Contains the basic system setup
///
/// Users of the app from this function should register all setup *BEFORE* the
/// [PostStartup] schedule and write an [AzStartupFailedEvent] or [AzStartupDryRunEvent]
/// if there is a need to exit after startup.
///
/// Users of the app from this function should also register their [Main] systems
/// to run only in [az_startup_succeeded] in order to benefit from early
/// termination if startup fails
///
pub fn bevy_app() -> App {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        TransformPlugin,
        DiagnosticsPlugin,
        HierarchyPlugin,
        AssetPlugin::default(),
    ))
    .init_state::<AzStartupState>()
    .add_event::<AzStartupFailedEvent>()
    .add_event::<AzStartupDryRunEvent>()
    .add_systems(PostStartup, check_startup_failures)
    .insert_resource(Time::<Fixed>::from_hz(DEFAULT_FRAME_RATE));

    app
}

fn check_startup_failures(
    current_state: Res<State<AzStartupState>>,
    mut next_state: ResMut<NextState<AzStartupState>>,
    mut ev_app_exit: EventWriter<AppExit>,
    mut ev_startup_failed: EventReader<AzStartupFailedEvent>,
    mut ev_startup_dryrun: EventReader<AzStartupDryRunEvent>,
) {
    let mut transiton_state = (**current_state).clone();
    for _ev in ev_startup_failed.read() {
        if transiton_state == AzStartupState::Succeeded {
            transiton_state = AzStartupState::Failed;
            error!("app startup failed! exiting");
            break;
        }
    }
    for _ev in ev_startup_dryrun.read() {
        if transiton_state == AzStartupState::Succeeded {
            transiton_state = AzStartupState::DryRun;
            info!("app startup detected a dry run! exiting");
            break;
        }
    }
    if transiton_state != AzStartupState::Succeeded {
        let exit = if matches!(transiton_state, AzStartupState::DryRun) {
            AppExit::Success
        } else {
            AppExit::error()
        };
        next_state.set(transiton_state);
        ev_app_exit.send(exit);
    }
}
