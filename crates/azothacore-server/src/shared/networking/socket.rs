use std::{fmt::Debug, future::Future, io, net::SocketAddr, pin::Pin, sync::Arc, time::Duration};

use azothacore_common::AzResult;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    runtime::Handle as TokioRuntimeHandler,
    sync::{mpsc, Mutex as AsyncMutex},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info_span, Instrument};

pub struct SocketWrappper {
    name:                 String,
    write_sender_channel: mpsc::Sender<bytes::Bytes>,
    read_stream:          AsyncMutex<Option<Pin<Box<dyn AsyncRead + Unpin + Send>>>>,
    write_join_handler:   Option<JoinHandle<()>>,
    rt_handler:           TokioRuntimeHandler,
    cancel_token:         CancellationToken,
}

impl Drop for SocketWrappper {
    fn drop(&mut self) {
        // cancels the socket and unsets the read receiver
        _ = self.rt_handler.block_on(self.close_socket());
        // Ensures that the write join handler is properly handled
        if let Some(jh) = self.write_join_handler.take() {
            if let Err(e) = self.rt_handler.block_on(jh) {
                error!(target:"network", name=self.name, cause=?e, "error joining on runtime when dropping socket");
            }
        }
    }
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
        let wr_ct = cancel_token.clone();
        let socket_write_span = info_span!(target: "network", "socket_write_span", name=name.to_string());
        let write_join_handler = rt_handler.spawn(
            async move {
                let mut flush_interval = tokio::time::interval(Duration::from_secs(1));
                flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                loop {
                    let mut recv_bytes_to_write = tokio::select! {
                        _ = wr_ct.cancelled() => {
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
                // Shutdown everything
                if let Err(e) = wr.shutdown().await {
                    error!(cause=?e, "shutdown write socket error: {e}");
                }
                wr_ct.cancel();
                wr_rcv.close();
            }
            .instrument(socket_write_span),
        );

        Self {
            name: name.to_string(),
            write_sender_channel,
            read_stream: AsyncMutex::new(Some(Box::pin(rd))),
            write_join_handler: Some(write_join_handler),
            rt_handler,
            cancel_token,
        }
    }

    /// Returns Ok if not closed, else Err if closed
    pub fn is_running(&self) -> io::Result<()> {
        if self.cancel_token.is_cancelled() {
            Err(io::Error::new(io::ErrorKind::ConnectionAborted, format!("{} is closed", self.name)))
        } else {
            Ok(())
        }
    }

    /// Close underlying socket by dropping everything
    /// Will always return an error if closed, otherwise returns Ok if not.
    pub async fn close_socket(&self) -> io::Result<()> {
        error!(target: "network", "closing socket due to error in receive/write or calls to close by request / server");
        self.cancel_token.cancel();
        *self.read_stream.lock().await = None;
        self.write_sender_channel.closed().await;
        self.is_running()
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
            _ = self.close_socket().await;
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
            _ = self.close_socket().await;
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
    fn new_from_tcp(
        rt_handler: TokioRuntimeHandler,
        cancel_token: CancellationToken,
        addr: AddressOrName,
        tcp_conn: TcpStream,
    ) -> impl Future<Output = AzResult<Arc<Self>>> + Send
    where
        Self: std::marker::Sized,
    {
        async {
            let (rd, wr) = tcp_conn.into_split();
            let rd = tokio::io::BufReader::new(rd);
            let wr = tokio::io::BufWriter::new(wr);
            Ok(Self::new(rt_handler, cancel_token, addr, rd, wr))
        }
    }

    fn new<R, W>(rt_handler: TokioRuntimeHandler, cancel_token: CancellationToken, name: AddressOrName, rd: R, wr: W) -> Arc<Self>
    where
        R: AsyncRead + Unpin + Send + Sync + 'static,
        W: AsyncWrite + Unpin + Send + Sync + 'static;

    fn start(&self) -> impl std::future::Future<Output = AzResult<()>> + Send;

    /// Returns Ok(()) if its still running, otherwise if it has already closed, will always return an error.
    fn is_running(&self) -> AzResult<()>;

    fn close(&self) -> impl std::future::Future<Output = AzResult<()>> + Send;
}
