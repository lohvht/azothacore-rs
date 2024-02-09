use std::{collections::VecDeque, future::Future, io, sync::Arc, time::Duration};

use azothacore_common::r#async::Context;
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    sync::Mutex as AsyncMutex,
};
use tracing::{debug, error};

use crate::shared::networking::socket::{AddressOrName, Socket};

pub trait SocketMgr<S>
where
    S: Socket + Sync + Send + 'static,
{
    fn socket_added(_s: Arc<S>) {}
    fn socket_removed(_s: Arc<S>) {}
    fn start_network_impl<A>(ctx: Context, bind_addr: A, all_sockets: Arc<AsyncMutex<Vec<Arc<S>>>>) -> impl Future<Output = io::Result<()>> + Send
    where
        A: ToSocketAddrs + Send,
    {
        async move {
            let acceptor = TcpListener::bind(bind_addr).await?;
            let new_sockets = Arc::new(AsyncMutex::new(VecDeque::new()));

            Self::start_async_accept(ctx.clone(), acceptor, new_sockets.clone());
            Self::start_async_socket_management(ctx.clone(), new_sockets, all_sockets);

            Ok(())
        }
    }

    #[tracing::instrument(skip_all, target = "network", name = "async_socket_update_span")]
    fn start_async_accept(ctx: Context, acceptor: TcpListener, new_sockets: Arc<AsyncMutex<VecDeque<Arc<S>>>>) {
        let ctx_cloned = ctx.clone();
        ctx.spawn(async move {
            loop {
                let (conn, addr) = tokio::select! {
                    _ = ctx_cloned.cancelled() => {
                        debug!("cancellation token set, terminating network async accept");
                        break;
                    }
                    // TODO: Introduce semaphore to limit number of connections
                    conn = acceptor.accept() => {
                        match conn {
                            Err(e) => {
                                error!(cause=?e, "failed to retrieve client connection");
                                continue
                            },
                            Ok(c) => c,
                        }
                    },
                };
                let sock = match S::start_from_tcp(ctx_cloned.child_token(), AddressOrName::Addr(addr), conn).await {
                    Err(e) => {
                        error!(cause=?e, name=%addr, "error creating socket / starting from TCP connection");
                        continue;
                    },
                    Ok(s) => s,
                };
                new_sockets.lock().await.push_back(sock);
            }
            ctx_cloned.cancel();
            new_sockets.lock().await.clear();
        });
    }

    #[tracing::instrument(skip_all, target = "network", name = "tcp_async_accept_span")]
    fn start_async_socket_management(ctx: Context, new_sockets: Arc<AsyncMutex<VecDeque<Arc<S>>>>, all_sockets: Arc<AsyncMutex<Vec<Arc<S>>>>) {
        let ctx_cloned = ctx.clone();
        ctx.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(10));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                let _t = tokio::select! {
                    _ = ctx_cloned.cancelled() => {
                        debug!("cancellation token set, terminating network socket mgmt");
                        break;
                    }
                    t = interval.tick() => t,
                };

                let mut all_sockets = all_sockets.lock().await;
                {
                    let mut new_sockets = new_sockets.lock().await;
                    while let Some(s) = new_sockets.pop_front() {
                        Self::socket_added(s.clone());
                        all_sockets.push(s);
                    }
                }
                all_sockets.retain(|s| {
                    let closed = s.is_closed();
                    if closed {
                        Self::socket_removed(s.clone());
                    }
                    !closed
                });
            }
            ctx_cloned.cancel();
            all_sockets.lock().await.clear();
            new_sockets.lock().await.clear();
        });
    }
}
