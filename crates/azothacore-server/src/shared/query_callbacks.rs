//! Async DB query callbacks, the bevy equivalent of TC's `QueryCallbackProcessor`
//! (i.e. `_queryProcessor.AddCallback(...)`).
//!
//! Queries are spawned onto the tokio runtime so that bevy systems never block on
//! them. Once a query completes, its callback is queued and then ran on the next
//! tick, either:
//!
//! - with exclusive world access in [process_query_callbacks] ([QueryCallbacks::add_callback]
//!   and friends), or
//! - as a command queued by the non-exclusive `process_query_command_callbacks`
//!   ([QueryCallbacks::add_commands_system_callback]), so no dedicated exclusive
//!   system is needed. The callback system still runs with exclusive world access,
//!   but at the next sync point, along with the other queued commands.
//!
//! Callbacks are ran in the order which their queries *complete*, not the order
//! in which they were added. If steps must happen in sequence, `.await` them
//! within the same future instead.

use std::future::Future;

use azothacore_common::bevy_app::TokioRuntime;
use bevy::prelude::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tracing::error;

type WorldCallback = Box<dyn FnOnce(&mut World) + Send>;
type CommandsCallback = Box<dyn FnOnce(&mut Commands) + Send>;

/// Handle used to queue async queries along with the callbacks to run once they
/// complete. Cheap to clone.
#[derive(Resource, Clone)]
pub struct QueryCallbacks {
    rt:          tokio::runtime::Handle,
    world_tx:    UnboundedSender<WorldCallback>,
    commands_tx: UnboundedSender<CommandsCallback>,
}

#[derive(Resource)]
struct QueryCallbackReceiver(UnboundedReceiver<WorldCallback>);

#[derive(Resource)]
struct QueryCommandsCallbackReceiver(UnboundedReceiver<CommandsCallback>);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcessQueryCallbacksSet;

impl QueryCallbacks {
    fn spawn<T, Fut, C>(&self, tx: &UnboundedSender<C>, fut: Fut, wrap: impl FnOnce(T) -> C + Send + 'static)
    where
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
        C: Send + 'static,
    {
        let tx = tx.clone();
        self.rt.spawn(async move {
            let res = fut.await;
            // Receiver is only dropped when the app is shutting down, dropping the callback then is fine.
            _ = tx.send(wrap(res));
        });
    }

    /// Runs `fut` on the tokio runtime, then `cb` with its output on a subsequent tick.
    pub fn add_callback<T, Fut, F>(&self, fut: Fut, cb: F)
    where
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
        F: FnOnce(T, &mut World) + Send + 'static,
    {
        self.spawn(&self.world_tx, fut, |res| -> WorldCallback { Box::new(move |world| cb(res, world)) });
    }

    /// Like [Self::add_callback], but the callback is a regular bevy system taking
    /// the future's output as its [In] parameter.
    ///
    /// The system must be zero-sized (i.e. a fn item or a non-capturing closure)
    /// as it is ran as a cached one-shot system. Pass any context the callback
    /// needs through the future's output instead of capturing it.
    pub fn add_system_callback<T, Fut, S, M>(&self, fut: Fut, system: S)
    where
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
        S: IntoSystem<In<T>, (), M> + Send + 'static,
    {
        self.add_callback(fut, move |res, world| {
            if let Err(e) = world.run_system_cached_with(system, res) {
                error!(cause=?e, "error running query callback system");
            }
        });
    }

    /// Like [Self::add_callback], but the callback is dropped if `entity` has been
    /// despawned by the time the query completes, e.g. a session that logged out.
    pub fn add_entity_callback<T, Fut, F>(&self, entity: Entity, fut: Fut, cb: F)
    where
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
        F: FnOnce(T, Entity, &mut World) + Send + 'static,
    {
        self.add_callback(fut, move |res, world| {
            if !world.entities().contains(entity) {
                return;
            }
            cb(res, entity, world);
        });
    }

    /// Like [Self::add_system_callback], but the system is ran through
    /// [Commands::run_system_cached_with], i.e. at the next sync point instead of
    /// in the exclusive [process_query_callbacks].
    ///
    /// The same zero-sized system restriction applies.
    pub fn add_commands_system_callback<T, Fut, S, M>(&self, fut: Fut, system: S)
    where
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
        S: IntoSystem<In<T>, (), M> + Send + 'static,
        M: 'static,
    {
        self.spawn(&self.commands_tx, fut, |res| -> CommandsCallback {
            Box::new(move |commands| commands.run_system_cached_with(system, res))
        });
    }
}

/// Runs the world callbacks of all queries that have completed so far.
pub fn process_query_callbacks(world: &mut World) {
    // Drain first so that callbacks queueing further callbacks do not get ran
    // within the same tick, avoiding an endless loop.
    let ready: Vec<_> = {
        let mut rcv = world.resource_mut::<QueryCallbackReceiver>();
        std::iter::from_fn(|| rcv.0.try_recv().ok()).collect()
    };
    for cb in ready {
        cb(world);
    }
}

/// Runs the commands callbacks of all queries that have completed so far.
fn process_query_command_callbacks(mut rcv: ResMut<QueryCommandsCallbackReceiver>, mut commands: Commands) {
    // The callback systems only run once the commands are applied, so draining
    // while queueing cannot loop endlessly.
    while let Ok(cb) = rcv.0.try_recv() {
        cb(&mut commands);
    }
}

/// Requires [TokioRuntime] to be inserted before this plugin is added.
pub fn query_callbacks_plugin(app: &mut App) {
    let rt = app
        .world()
        .get_resource::<TokioRuntime>()
        .expect("TokioRuntime must be inserted before query_callbacks_plugin")
        .handle()
        .clone();
    let (world_tx, world_rcv) = unbounded_channel();
    let (commands_tx, commands_rcv) = unbounded_channel();
    app.insert_resource(QueryCallbacks { rt, world_tx, commands_tx })
        .insert_resource(QueryCallbackReceiver(world_rcv))
        .insert_resource(QueryCommandsCallbackReceiver(commands_rcv))
        .add_systems(
            FixedUpdate,
            (process_query_callbacks, process_query_command_callbacks).in_set(ProcessQueryCallbacksSet),
        );
}
