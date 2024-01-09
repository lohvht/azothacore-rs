use std::{fmt::Debug, future::Future, io, net::SocketAddr, pin::Pin, sync::Arc, time::Duration};

use azothacore_common::AzResult;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    runtime::Handle as TokioRuntimeHandler,
    sync::{mpsc, Mutex as AsyncMutex},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info_span, Instrument};

use crate::shared::{dropper_wrapper_fn, DropperWrapperFn};

type AsyncSharedRead = Arc<AsyncMutex<Option<Pin<Box<dyn AsyncRead + Unpin + Send>>>>>;

pub struct SocketWrappper {
    name:                 String,
    write_sender_channel: mpsc::Sender<bytes::Bytes>,
    read_stream:          AsyncSharedRead,
    cancel_token:         CancellationToken,
    _task:                DropperWrapperFn,
}

/// SocketWrappper provides a socket wrapper that is cancel safe, with reads/writes not being
/// mutable (via channels and interior mutability w/ mutexes)
impl SocketWrappper {
    pub fn new<R, W>(rt_handler: TokioRuntimeHandler, cancel_token: CancellationToken, name: AddressOrName, rd: R, mut wr: W) -> Self
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let (write_sender_channel, mut wr_rcv) = mpsc::channel(1024);
        let ct = cancel_token.clone();
        let read_stream: AsyncSharedRead = Arc::new(AsyncMutex::new(Some(Box::pin(rd))));

        let socket_write_span = info_span!(target: "network", "socket_write_span", name=name.to_string());
        let read_stream_clone = read_stream.clone();
        let write_sender_channel_clone = write_sender_channel.clone();
        let _task = rt_handler.spawn(
            async move {
                let mut flush_interval = tokio::time::interval(Duration::from_secs(1));
                flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                loop {
                    let mut recv_bytes_to_write = tokio::select! {
                        _ = ct.cancelled() => {
                            debug!("shutdown write socket due to token cancellation");
                            break;
                        },
                        _ = flush_interval.tick() => match wr.flush().await {
                            Err(e) if e.kind() != io::ErrorKind::WriteZero => {
                                error!(cause=?e, "shutdown as encountered non recoverable error when flushing buffers");
                                break;
                            },
                            _ => continue,
                        },
                        opt_by = wr_rcv.recv() => {
                            match opt_by {
                                None => {
                                    error!("shutdown write socket to due to write receiver channel closing");
                                    break;
                                },
                                Some(b) => b,
                            }
                        },
                    };
                    if let Err(e) = wr.write_all_buf(&mut recv_bytes_to_write).await {
                        error!(cause=?e, "shutdown write socket to due to write receiver channel closing");
                        break;
                    }
                }
                // Shutdown everything, cleaning up
                if let Err(e) = wr.shutdown().await {
                    error!(cause=?e, "shutdown write socket error: {e}");
                }
                // doesn't hurt to cancel another time anyway
                ct.cancel();
                // drop receiver, this should make the sender execute immediately too
                drop(wr_rcv);
                // drop read stream, this should have the effect of closing the read half of the stream
                *read_stream_clone.lock().await = None;
                // wait for sender to close, this should also run immediately
                write_sender_channel_clone.closed().await;
            }
            .instrument(socket_write_span),
        );

        let _task = dropper_wrapper_fn(&rt_handler, cancel_token.clone(), async { Ok(_task.await?) });
        Self {
            name: name.to_string(),
            write_sender_channel,
            read_stream,
            cancel_token,
            _task,
        }
    }

    /// Returns Ok if not closed, else Err if closed
    pub fn is_closed(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    pub async fn wait_closed(&self) {
        // token has been cancelled
        self.cancel_token.cancelled().await;

        // write channel must also be closed
        self.write_sender_channel.closed().await;
    }

    pub fn close_socket(&self) {
        self.cancel_token.cancel();
    }

    pub async fn receive(&self, num_bytes: usize) -> io::Result<bytes::Bytes> {
        let mut read_stream_lock = self.read_stream.lock().await;
        let read_stream = match read_stream_lock.as_mut() {
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    format!("{} received failed due to already closed read connection", self.name),
                ))
            },
            Some(s) => s,
        };
        let mut buf = vec![0u8; num_bytes];
        let res = tokio::select! {
            _ = self.cancel_token.cancelled() => Err(io::Error::new(io::ErrorKind::ConnectionAborted, format!("{} received failed due to already cancelled connection", self.name))),
            r = read_stream.read_exact(&mut buf) => r,
        };

        if let Err(e) = &res {
            error!(target: "network", cause=%e, "receive error, closing socket");
            self.close_socket();
        }

        res.map(|_| buf.into())
    }

    pub async fn write(&self, b: impl Into<bytes::Bytes>) -> io::Result<()> {
        let res = tokio::select! {
            _ = self.cancel_token.cancelled() => Err(io::Error::new(io::ErrorKind::ConnectionAborted, format!("{} send write failed due to already cancelled connection", self.name))),
            r = self.write_sender_channel.send(b.into()) => r.map_err(|e| {
                io::Error::new(io::ErrorKind::ConnectionAborted, format!("{} send write failed due to already closed receiver, err={e}", self.name))
            }),
        };

        if let Err(e) = &res {
            error!(target: "network", cause=%e, "write error, closing socket");
            self.close_socket();
        }
        res
    }
}

#[derive(Clone)]
pub enum AddressOrName {
    Addr(SocketAddr),
    Name(String),
}

impl AddressOrName {
    pub fn ip_str_or_name(&self) -> String {
        match self {
            AddressOrName::Addr(a) => a.ip().to_string(),
            AddressOrName::Name(s) => s.clone(),
        }
    }
}

impl From<SocketAddr> for AddressOrName {
    fn from(value: SocketAddr) -> Self {
        AddressOrName::Addr(value)
    }
}

impl From<String> for AddressOrName {
    fn from(value: String) -> Self {
        AddressOrName::Name(value)
    }
}

impl Debug for AddressOrName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressOrName::Addr(addr) => Debug::fmt(addr, f),
            AddressOrName::Name(s) => Debug::fmt(s, f),
        }
    }
}

impl std::fmt::Display for AddressOrName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressOrName::Addr(addr) => std::fmt::Display::fmt(addr, f),
            AddressOrName::Name(s) => std::fmt::Display::fmt(s, f),
        }
    }
}

pub trait Socket {
    fn start_from_tcp(cancel_token: CancellationToken, addr: AddressOrName, tcp_conn: TcpStream) -> impl Future<Output = AzResult<Arc<Self>>> + Send
    where
        Self: std::marker::Sized,
    {
        async {
            let (rd, wr) = tcp_conn.into_split();
            let rd = tokio::io::BufReader::new(rd);
            let wr = tokio::io::BufWriter::new(wr);
            Self::start(cancel_token, addr, rd, wr).await
        }
    }

    fn start<R, W>(cancel_token: CancellationToken, name: AddressOrName, rd: R, wr: W) -> impl Future<Output = AzResult<Arc<Self>>> + Send
    where
        R: AsyncRead + Unpin + Send + Sync + 'static,
        W: AsyncWrite + Unpin + Send + Sync + 'static;

    fn is_closed(&self) -> bool;

    fn wait_closed(&self) -> impl std::future::Future<Output = ()> + Send;

    fn close(&self);
}
