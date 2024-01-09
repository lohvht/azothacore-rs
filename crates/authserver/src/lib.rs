#![feature(lint_reasons)]

use std::{io, sync::Arc};

use azothacore_server::shared::networking::socket_mgr::SocketMgr;
use bnet_rpc::BnetRpcService;
use tokio::{
    net::ToSocketAddrs,
    runtime,
    sync::{Mutex as AsyncMutex, RwLock as AsyncRwLock},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

pub mod rest;
pub mod session;
pub mod ssl_context;

pub struct BnetSessionManager {
    cancel_token: AsyncRwLock<Option<CancellationToken>>,
    tasks:        AsyncRwLock<Option<JoinSet<()>>>,
}

impl SocketMgr<session::Session> for BnetSessionManager {
    fn socket_removed(s: Arc<session::Session>) {
        debug!(target:"session", caller_info=s.caller_info(), "Socket removed!");
    }

    async fn stop_network(&self) -> std::io::Result<()> {
        if let Some(ct) = self.cancel_token.write().await.take() {
            ct.cancel();
        }
        // Ensures that the write join handler is properly handled
        if let Some(mut js) = self.tasks.write().await.take() {
            while let Some(res) = js.join_next().await {
                if let Err(e) = res {
                    error!(target:"session", cause=?e, "error joining on runtime and stopping accept task when dropping socket mgr");
                }
            }
        };
        Ok(())
    }
}

impl BnetSessionManager {
    pub const fn new() -> Self {
        Self {
            cancel_token: AsyncRwLock::const_new(None),
            tasks:        AsyncRwLock::const_new(None),
        }
    }

    pub async fn stop_network() -> io::Result<()> {
        BNET_SESSION_MGR.stop_network().await
    }

    pub fn start_network<A>(rt_h: &runtime::Handle, cancel_token: CancellationToken, bind_addr: A) -> io::Result<()>
    where
        A: ToSocketAddrs + Send,
    {
        let all_sockets = Arc::new(AsyncMutex::new(vec![]));
        let rt_h = rt_h.clone();
        let rt_handler = rt_h.clone();
        let ct = cancel_token.clone();
        let tasks = rt_handler
            .block_on(async { Self::start_network_impl(all_sockets, cancel_token, bind_addr).await })
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("error starting bnet session manager: {e}")))?;

        *BNET_SESSION_MGR.tasks.blocking_write() = Some(tasks);
        *BNET_SESSION_MGR.cancel_token.blocking_write() = Some(ct);

        Ok(())
    }
}

static BNET_SESSION_MGR: BnetSessionManager = BnetSessionManager::new();
