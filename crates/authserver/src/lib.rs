#![feature(lint_reasons)]

use std::{io, sync::Arc};

use azothacore_common::r#async::Context;
use azothacore_server::shared::networking::socket_mgr::SocketMgr;
use bnet_rpc::BnetRpcService;
use tokio::{net::ToSocketAddrs, sync::Mutex as AsyncMutex};
use tracing::{debug, error, info};

pub mod rest;
pub mod session;
pub mod ssl_context;

pub struct BnetSessionManager;

impl SocketMgr<session::Session> for BnetSessionManager {
    fn socket_removed(s: Arc<session::Session>) {
        debug!(target:"session", caller_info=s.caller_info(), "Socket removed!");
    }
}

impl BnetSessionManager {
    pub async fn start_network<A>(ctx: Context, bind_addr: A) -> io::Result<()>
    where
        A: ToSocketAddrs + Send,
    {
        let all_sockets = Arc::new(AsyncMutex::new(vec![]));
        if let Err(e) = Self::start_network_impl(ctx.clone(), bind_addr, all_sockets).await {
            error!(target:"session", cause=%e, "error starting bnet session manager");
            ctx.cancel();
        }

        ctx.cancelled().await;
        info!(target:"session", "terminating bnet session manager!");

        Ok(())
    }
}
