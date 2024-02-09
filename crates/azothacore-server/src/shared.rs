pub mod data_stores;
pub mod networking;
pub mod realms;
pub mod secrets;
pub mod shared_defines;

use std::{
    io::{self, Write},
    panic,
};

use azothacore_common::r#async::Context;
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

pub async fn signal_handler(ctx: Context) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    let mut sig_break = match tokio::signal::windows::ctrl_break::ctrl_break() {
        Err(e) => {
            ctx.cancel();
            info!(cause=%e, "Unable to init signal handler: sig_break");
            return Err(e);
        },
        Ok(s) => s,
    };
    #[cfg(target_os = "linux")]
    let mut sig_interrupt = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()) {
        Err(e) => {
            ctx.cancel();
            info!(cause=%e, "Unable to init signal handler: sig_interrupt");
            return Err(e);
        },
        Ok(s) => s,
    };
    #[cfg(target_os = "linux")]
    let mut sig_terminate = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
        Err(e) => {
            ctx.cancel();
            info!(cause=%e, "Unable to init signal handler: sig_terminate");
            return Err(e);
        },
        Ok(s) => s,
    };
    #[cfg(target_os = "linux")]
    let mut sig_quit = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::quit()) {
        Err(e) => {
            ctx.cancel();
            info!(cause=%e, "Unable to init signal handler: sig_quit");
            return Err(e);
        },
        Ok(s) => s,
    };
    #[cfg(target_os = "windows")]
    tokio::select! {
        _ = ctx.cancelled() => {
            info!("Token cancelled, terminating signal handler");
        },
        _ = sig_break.recv() => {
            info!("Received signal SIGBREAK");
        },
    };
    #[cfg(target_os = "linux")]
    tokio::select! {
        _ = ctx.cancelled() => {
            info!("Token cancelled, terminating signal handler");
        },
        _ = sig_interrupt.recv() => {
            info!("Received signal SIGINT");
        },
        _ = sig_terminate.recv() => {
            info!("Received signal SIGTERM");
        },
        _ = sig_quit.recv() => {
            info!("Received signal SIGQUIT");
        },
    };
    info!("Terminating due to receiving a stop signal");
    ctx.cancel();

    Ok(())
}
