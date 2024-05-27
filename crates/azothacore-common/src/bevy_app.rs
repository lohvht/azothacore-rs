use bevy::{app::ScheduleRunnerPlugin, diagnostic::DiagnosticsPlugin, prelude::*, time::TimePlugin};
use tokio::runtime::Runtime;

use crate::deref_boilerplate;

/// Default we use for FixedUpdates in azothacore, can be overwritten again
const DEFAULT_FRAME_RATE: f64 = 144.0; // Bevy default 64;

/// A newtype that implements bevy resource that wraps around a tokio runtime
#[derive(Resource)]
pub struct TokioRuntime(Runtime);

deref_boilerplate!(TokioRuntime, Runtime, 0);

pub fn bevy_app() -> App {
    let mut app = App::new();
    let tokio_rt = TokioRuntime(tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap());

    app.add_plugins(TypeRegistrationPlugin)
        .add_plugins(FrameCountPlugin)
        .add_plugins(TimePlugin)
        .add_plugins(ScheduleRunnerPlugin::default())
        .add_plugins(TransformPlugin)
        .add_plugins(DiagnosticsPlugin)
        .add_plugins(HierarchyPlugin)
        .insert_resource(Time::<Fixed>::from_hz(DEFAULT_FRAME_RATE))
        .insert_resource(tokio_rt);

    app
}
