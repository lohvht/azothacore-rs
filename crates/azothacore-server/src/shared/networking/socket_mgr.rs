use std::{collections::VecDeque, future::Future, io, sync::Arc, time::Duration};

use tokio::{
    net::{TcpListener, ToSocketAddrs},
    runtime::Handle as TokioRuntimeHandler,
    sync::Mutex as AsyncMutex,
    task::JoinHandle,
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
        rt_h: TokioRuntimeHandler,
        cancel_token: CancellationToken,
        bind_addr: A,
    ) -> impl Future<Output = io::Result<(JoinHandle<()>, JoinHandle<()>)>> + Send
    where
        A: ToSocketAddrs + Send,
    {
        async move {
            let acceptor = TcpListener::bind(bind_addr).await?;
            let new_sockets = Arc::new(AsyncMutex::new(VecDeque::new()));

            let async_accept_task = Self::start_async_accept(&rt_h, cancel_token.clone(), acceptor, new_sockets.clone());
            let async_socket_update_task = Self::start_async_socket_management(&rt_h, cancel_token.clone(), new_sockets, all_sockets);

            Ok((async_accept_task, async_socket_update_task))
        }
    }

    fn stop_network(&mut self) -> io::Result<()>;

    #[tracing::instrument(skip_all, target = "network", name = "async_socket_update_span")]
    fn start_async_accept(
        rt_h: &TokioRuntimeHandler,
        cancel_token: CancellationToken,
        acceptor: TcpListener,
        new_sockets: Arc<AsyncMutex<VecDeque<Arc<S>>>>,
    ) -> JoinHandle<()> {
        let rth_clone = rt_h.clone();
        rt_h.spawn(async move {
            let rth_clone = rth_clone;
            loop {
                let (conn, addr) = tokio::select! {
                    _ = cancel_token.cancelled() => {
                        debug!("cancellation token set, terminating network");
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
                let sock = match S::new_from_tcp(rth_clone.clone(), cancel_token.child_token(), AddressOrName::Addr(addr), conn).await {
                    Err(e) => {
                        error!(cause=?e, name=%addr, "error creating socket from TCP connection");
                        continue;
                    },
                    Ok(s) => s,
                };
                // similar to running OnSocketAccept in TC/AC
                if let Err(e) = sock.start().await {
                    error!(cause=?e, name=%addr, "failed to start client connection");
                    continue;
                }
                new_sockets.lock().await.push_back(sock);
            }
            cancel_token.cancel();
            new_sockets.lock().await.clear();
        })
    }

    #[tracing::instrument(skip_all, target = "network", name = "tcp_async_accept_span")]
    fn start_async_socket_management(
        rt_h: &TokioRuntimeHandler,
        cancel_token: CancellationToken,
        new_sockets: Arc<AsyncMutex<VecDeque<Arc<S>>>>,
        all_sockets: Arc<AsyncMutex<Vec<Arc<S>>>>,
    ) -> JoinHandle<()> {
        rt_h.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(10));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                let _t = tokio::select! {
                    _ = cancel_token.cancelled() => {
                        debug!("cancellation token set, terminating network");
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
                    let r = s.is_running();
                    if r.is_err() {
                        Self::socket_removed(s.clone());
                    }
                    r.is_ok()
                });
            }
            cancel_token.cancel();
            all_sockets.lock().await.clear();
            new_sockets.lock().await.clear();
        })
    }
}
