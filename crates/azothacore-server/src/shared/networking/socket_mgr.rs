use std::{collections::VecDeque, future::Future, io, sync::Arc, time::Duration};

use tokio::{
    net::{TcpListener, ToSocketAddrs},
    sync::Mutex as AsyncMutex,
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

use crate::shared::networking::socket::{AddressOrName, Socket};

pub trait SocketMgr<S>
where
    S: Socket + Sync + Send + 'static,
{
    fn socket_added(_s: Arc<S>) {}
    fn socket_removed(_s: Arc<S>) {}
    fn start_network_impl<A>(
        all_sockets: Arc<AsyncMutex<Vec<Arc<S>>>>,
        cancel_token: CancellationToken,
        bind_addr: A,
    ) -> impl Future<Output = io::Result<JoinSet<()>>> + Send
    where
        A: ToSocketAddrs + Send,
    {
        async move {
            let acceptor = TcpListener::bind(bind_addr).await?;
            let new_sockets = Arc::new(AsyncMutex::new(VecDeque::new()));

            let mut joinset = JoinSet::new();
            Self::start_async_accept(&mut joinset, cancel_token.clone(), acceptor, new_sockets.clone());
            Self::start_async_socket_management(&mut joinset, cancel_token.clone(), new_sockets, all_sockets);

            Ok(joinset)
        }
    }

    fn stop_network(&self) -> impl Future<Output = io::Result<()>> + Send;

    #[tracing::instrument(skip_all, target = "network", name = "async_socket_update_span")]
    fn start_async_accept(joinset: &mut JoinSet<()>, cancel_token: CancellationToken, acceptor: TcpListener, new_sockets: Arc<AsyncMutex<VecDeque<Arc<S>>>>) {
        joinset.spawn(async move {
            loop {
                let (conn, addr) = tokio::select! {
                    _ = cancel_token.cancelled() => {
                        debug!("cancellation token set, terminating network async accept");
                        break;
                    }
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
                let sock = match S::start_from_tcp(cancel_token.child_token(), AddressOrName::Addr(addr), conn).await {
                    Err(e) => {
                        error!(cause=?e, name=%addr, "error creating socket / starting from TCP connection");
                        continue;
                    },
                    Ok(s) => s,
                };
                new_sockets.lock().await.push_back(sock);
            }
            cancel_token.cancel();
            new_sockets.lock().await.clear();
        });
    }

    #[tracing::instrument(skip_all, target = "network", name = "tcp_async_accept_span")]
    fn start_async_socket_management(
        joinset: &mut JoinSet<()>,
        cancel_token: CancellationToken,
        new_sockets: Arc<AsyncMutex<VecDeque<Arc<S>>>>,
        all_sockets: Arc<AsyncMutex<Vec<Arc<S>>>>,
    ) {
        joinset.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(10));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                let _t = tokio::select! {
                    _ = cancel_token.cancelled() => {
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
            cancel_token.cancel();
            all_sockets.lock().await.clear();
            new_sockets.lock().await.clear();
        });
    }
}
