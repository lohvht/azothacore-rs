pub mod data_stores;
pub mod networking;
pub mod realms;
pub mod secrets;
pub mod shared_defines;

use std::{
    io::{self, Write},
    panic,
};

use azothacore_common::{bevy_app::TokioRuntime, r#async::Context};
use bevy::{app::AppExit, prelude::*, tasks::poll_once};
use thiserror::Error;
use tokio::task::JoinHandle;
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

pub fn panic_handler(ctx: Context) {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        error!(target:"server", "panic received! start termination");
        ctx.cancel();
        original_hook(panic_info);
    }));
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

pub async fn signal_handler() -> Result<String, SignalError> {
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
            "SIGBREAK".into()
        },
    };
    #[cfg(target_os = "linux")]
    let sig = tokio::select! {
        _ = sig_interrupt.recv() => {
            "SIGINT".into()
        },
        _ = sig_terminate.recv() => {
            "SIGTERM".into()
        },
        _ = sig_quit.recv() => {
            "SIGQUIT".into()
        },
    };

    Ok(sig)
}

pub fn tokio_signal_handling_bevy_plugin(app: &mut App) {
    app.add_systems(Startup, overwrite_signal_handlers).add_systems(FixedUpdate, try_receive_signal);
}

#[derive(Component)]
struct SignalHandlerTokioTask(JoinHandle<Result<String, SignalError>>);

fn overwrite_signal_handlers(mut commands: Commands, rt: Res<TokioRuntime>) {
    let task = rt.spawn(async move { signal_handler().await });
    let entity = commands.spawn_empty().id();
    commands.entity(entity).insert(SignalHandlerTokioTask(task));
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
    app_exit_events.send(AppExit);
    // Not entirely necessary but this ensure that if the app has already ran the
    // app exit, we will forcefully panic instead.
    commands.entity(e).remove::<SignalHandlerTokioTask>();
}
