use std::ops::{Deref, DerefMut};

use bevy::{
    asset::AssetPlugin,
    diagnostic::DiagnosticsPlugin,
    ecs::query::QueryEntityError,
    prelude::{
        App,
        AppExit,
        Bundle,
        Entity,
        EntityCommands,
        Event,
        EventReader,
        EventWriter,
        Fixed,
        HierarchyPlugin,
        Mut,
        NextState,
        PostStartup,
        Res,
        ResMut,
        Resource,
        Time,
        TransformPlugin,
    },
    state::{
        app::{AppExtStates, StatesPlugin},
        condition::in_state,
        state::{State, States},
    },
    MinimalPlugins,
};
use tokio::runtime::Runtime;
use tracing::{error, info};

use crate::{az_error, deref_boilerplate, AzResult};

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

pub trait ToFromEntity {
    fn from_entity(entity: Entity) -> Self;
    fn to_entity(self) -> Entity;
}

/// Primarily handles the error where the component is not found
///
/// This encapsulates the logic for handling [QueryDoesNotMatch] check
/// by turning it into a [None]
pub fn query_not_found_result<V>(res: Result<V, QueryEntityError>) -> AzResult<Option<V>> {
    res.map_or_else(
        |err| match err {
            QueryEntityError::QueryDoesNotMatch(..) => Ok(None),
            err => Err(az_error!("unable to retrieve from query: type V was: {}", std::any::type_name::<V>()).context(format!("original error: {err}"))),
        },
        |v| Ok(Some(v)),
    )
}

pub enum QueryOrNewSingleMut<'w, V: Bundle> {
    Existing(Mut<'w, V>),
    New(Option<V>, EntityCommands<'w>),
}

// impl<'w, 's, V: Bundle> QueryOrNewSingleMut<'w, 's, V> {
//     pub fn query<D, F, N>(query: &mut Query<'w, 's, D, F>, entity_id: Entity, new: N) -> Result<Self, QueryEntityError<'w>>
//     where
//         D: QueryData<Item<'w> = Mut<'w, V>>,
//         F: QueryFilter,
//         N: FnOnce() -> (V, Commands<'w, 's>),
//     {
//         query.get_mut(entity_id).map_or_else(
//             move |err| match err {
//                 QueryEntityError::QueryDoesNotMatch(e, ..) => {
//                     let (v, cmd) = new();
//                     Ok(QueryOrNewSingleMut::New(Some(v), cmd, e))
//                 },
//                 err => Err(err),
//             },
//             move |v| Ok(QueryOrNewSingleMut::Existing(v)),
//         )

//         // match query.get_mut(entity_id) {
//         //     Ok(v) => Ok(QueryOrNewSingleMut::Existing(v)),
//         //     Err(QueryEntityError::QueryDoesNotMatch(e, ..)) => {
//         //         let (v, cmd) = new();
//         //         Ok(QueryOrNewSingleMut::New(Some(v), cmd, e))
//         //     },
//         //     Err(e) => Err(e),
//         // }
//     }
// }

impl<V: Bundle> Deref for QueryOrNewSingleMut<'_, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        match self {
            QueryOrNewSingleMut::Existing(v) => v,
            QueryOrNewSingleMut::New(v, ..) => v.as_ref().expect("This should never be unset"),
        }
    }
}

impl<V: Bundle> DerefMut for QueryOrNewSingleMut<'_, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            QueryOrNewSingleMut::Existing(v) => &mut *v,
            QueryOrNewSingleMut::New(v, ..) => v.as_mut().expect("This should never be unset"),
        }
    }
}

impl<V: Bundle> Drop for QueryOrNewSingleMut<'_, V> {
    fn drop(&mut self) {
        if let QueryOrNewSingleMut::New(v, ec) = self {
            let Some(v) = v.take() else {
                return;
            };
            ec.insert_if_new(v);
        }
    }
}
