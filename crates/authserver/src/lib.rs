#![feature(lint_reasons)]

use std::{
    io,
    sync::{Arc, OnceLock, RwLock, RwLockReadGuard},
};

use azothacore_common::{get_g, mut_g};
use azothacore_server::shared::networking::socket_mgr::SocketMgr;
use tokio::{net::ToSocketAddrs, runtime, sync::Mutex as AsyncMutex, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::error;

pub mod rest;
pub mod session;
pub mod ssl_context;

pub struct BnetSessionManager {
    rt_handler:         OnceLock<runtime::Handle>,
    cancel_token:       OnceLock<CancellationToken>,
    accept_task:        OnceLock<JoinHandle<()>>,
    socket_update_task: OnceLock<JoinHandle<()>>,
}

impl SocketMgr<session::Session> for BnetSessionManager {
    fn stop_network(&mut self) -> std::io::Result<()> {
        if let Some(ct) = self.cancel_token.get() {
            ct.cancel();
        }
        if let Some(rt) = self.rt_handler.take() {
            // Ensures that the write join handler is properly handled
            if let Some(jh) = self.accept_task.take() {
                if let Err(e) = rt.block_on(jh) {
                    error!(target:"session", cause=?e, "error joining on runtime and stopping accept task when dropping socket mgr");
                }
            };
            if let Some(jh) = self.socket_update_task.take() {
                if let Err(e) = rt.block_on(jh) {
                    error!(target:"session", cause=?e, "error joining on runtime and stopping update task when dropping socket mgr");
                }
            };
        }
        Ok(())
    }
}

impl BnetSessionManager {
    pub const fn new() -> Self {
        Self {
            accept_task:        OnceLock::new(),
            cancel_token:       OnceLock::new(),
            rt_handler:         OnceLock::new(),
            socket_update_task: OnceLock::new(),
        }
    }

    pub fn r() -> RwLockReadGuard<'static, Self> {
        get_g!(BNET_SESSION_MGR)
    }

    pub async fn stop_network() -> io::Result<()> {
        mut_g!(BNET_SESSION_MGR).stop_network()
    }

    pub fn start_network<A>(rt_h: &runtime::Handle, cancel_token: CancellationToken, bind_addr: A) -> io::Result<()>
    where
        A: ToSocketAddrs + Send,
    {
        let all_sockets = Arc::new(AsyncMutex::new(vec![]));
        let rt_h = rt_h.clone();
        let rt_handler = rt_h.clone();
        let ct = cancel_token.clone();
        let (accept_task, socket_update_task) = rt_handler
            .block_on(async { Self::start_network_impl(all_sockets, rt_h, cancel_token, bind_addr).await })
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("error starting bnet session manager: {e}")))?;

        let bnet_session_mgr = Self::r();
        bnet_session_mgr.accept_task.set(accept_task).unwrap();
        bnet_session_mgr.socket_update_task.set(socket_update_task).unwrap();
        bnet_session_mgr.rt_handler.set(rt_handler).unwrap();
        bnet_session_mgr.cancel_token.set(ct).unwrap();
        Ok(())
    }
}

static BNET_SESSION_MGR: RwLock<BnetSessionManager> = RwLock::new(BnetSessionManager::new());
