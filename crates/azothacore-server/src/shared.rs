pub mod data_stores;
pub mod networking;
pub mod realms;
pub mod secrets;
pub mod shared_defines;

use std::{
    io::{self, Write},
    panic,
};

use azothacore_common::bevy_app::TokioRuntime;
use bevy::{app::AppExit, prelude::*, tasks::poll_once};
use thiserror::Error;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tracing::{error, info};

pub fn bnetrpc_zcompress(mut json: Vec<u8>) -> io::Result<Vec<u8>> {
    use flate2::{write::ZlibEncoder, Compression};
    json.push(b'\0');
    let mut compressed = u32::try_from(json.len()).unwrap().to_le_bytes().to_vec();

    // Starts with the total eventual size
    let mut e = ZlibEncoder::new(vec![], Compression::default());
    e.write_all(&json)?;
    let mut res = e.finish()?;
    compressed.append(&mut res);
    Ok(compressed)
}

#[derive(Error, Debug)]
pub enum SignalError {
    #[error("unable to init signal handler {signal_name}: err={source}")]
    RegistrationError { signal_name: String, source: io::Error },
}

impl SignalError {
    fn reg_error(signal_name: &str, source: io::Error) -> Self {
        Self::RegistrationError {
            signal_name: signal_name.into(),
            source,
        }
    }
}

pub async fn signal_handler(signal_broadcast_snd: UnboundedSender<String>) -> Result<String, SignalError> {
    #[cfg(target_os = "windows")]
    let mut sig_break = match tokio::signal::windows::ctrl_break::ctrl_break() {
        Err(e) => {
            return Err(SignalError::reg_error("sig_break", e));
        },
        Ok(s) => s,
    };
    #[cfg(target_os = "linux")]
    let mut sig_interrupt = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()) {
        Err(e) => {
            return Err(SignalError::reg_error("sig_interrupt", e));
        },
        Ok(s) => s,
    };
    #[cfg(target_os = "linux")]
    let mut sig_terminate = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
        Err(e) => {
            return Err(SignalError::reg_error("sig_terminate", e));
        },
        Ok(s) => s,
    };
    #[cfg(target_os = "linux")]
    let mut sig_quit = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::quit()) {
        Err(e) => {
            return Err(SignalError::reg_error("sig_quit", e));
        },
        Ok(s) => s,
    };
    #[cfg(target_os = "windows")]
    let sig = tokio::select! {
        _ = sig_break.recv() => {
            "SIGBREAK".to_string()
        },
    };
    #[cfg(target_os = "linux")]
    let sig = tokio::select! {
        _ = sig_interrupt.recv() => {
            "SIGINT".to_string()
        },
        _ = sig_terminate.recv() => {
            "SIGTERM".to_string()
        },
        _ = sig_quit.recv() => {
            "SIGQUIT".to_string()
        },
    };
    _ = signal_broadcast_snd.send(sig.clone());

    Ok(sig)
}

pub fn tokio_signal_handling_bevy_plugin(app: &mut App) {
    app.add_systems(PreStartup, overwrite_signal_handlers)
        .add_systems(FixedUpdate, try_receive_signal);
}

#[derive(Component)]
struct SignalHandlerTokioTask(JoinHandle<Result<String, SignalError>>);

#[derive(Resource)]
pub struct SignalReceiver(pub UnboundedReceiver<String>);

fn overwrite_signal_handlers(mut commands: Commands, rt: Res<TokioRuntime>) {
    let (snd, rcv) = unbounded_channel();
    let task = rt.spawn(async move { signal_handler(snd).await });
    let entity = commands.spawn_empty().id();
    commands.entity(entity).insert(SignalHandlerTokioTask(task));
    commands.insert_resource(SignalReceiver(rcv));
}

fn try_receive_signal(
    mut commands: Commands,
    rt: Res<TokioRuntime>,
    mut tasks: Query<(Entity, &mut SignalHandlerTokioTask)>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    // anticipate only a single task, will panic if the app doesnt exit by then
    let (e, mut task) = tasks.get_single_mut().unwrap();
    let Some(res) = rt.block_on(poll_once(&mut task.0)) else {
        // Has yet to be been signalled
        return;
    };
    match res {
        Err(join_error) => {
            // If the task call was cancelled => should never be possible
            error!(cause=%join_error, "task cancelled before completion or error joining");
        },
        Ok(Err(signal_registration_err)) => {
            error!(cause=%signal_registration_err, "error init signal handler");
        },
        Ok(Ok(sig)) => {
            info!(signal = sig, "Terminating due to receiving a stop signal");
        },
    };
    app_exit_events.send(AppExit::Success);
    // Not entirely necessary but this ensure that if the app has already ran the
    // app exit, we will forcefully panic instead.
    commands.entity(e).remove::<SignalHandlerTokioTask>();
}
