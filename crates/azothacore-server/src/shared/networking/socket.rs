use std::{
    fmt::Debug,
    io,
    net::SocketAddr,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use azothacore_common::utils::{BufferDecodeError, DecodeValueFromBytes, MessageBuffer};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    runtime::Handle,
    sync::mpsc,
};
use tracing::{debug, error, instrument, warn};

pub struct Socket<P: DecodeValueFromBytes> {
    name:              AddressOrName,
    snd_write_packets: mpsc::UnboundedSender<bytes::Bytes>,
    rcv_read_packets:  mpsc::UnboundedReceiver<P>,
    exited_flag:       Arc<AtomicBool>,
    exiting_flag:      AtomicBool,
    snd_exiting:       mpsc::UnboundedSender<()>,
}

#[derive(Debug)]
pub enum SocketStatus {
    Running,
    Closing,
    Closed,
}

impl SocketStatus {
    fn is_running(&self) -> bool {
        matches!(self, SocketStatus::Running)
    }
}

impl<P: DecodeValueFromBytes + Send + 'static> Socket<P> {
    pub fn new<R, W>(tokio_handler: &Handle, name: AddressOrName, rd: R, wr: W) -> Self
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let exited_flag = Arc::new(AtomicBool::new(false));
        let (snd_write_packets, rcv_write_packets) = mpsc::unbounded_channel();
        let (snd_read_packets, rcv_read_packets) = mpsc::unbounded_channel();
        let (snd_exiting, rcv_exiting) = mpsc::unbounded_channel();
        tokio_handler.spawn(handle_socket(
            name.to_string(),
            rd,
            wr,
            rcv_write_packets,
            snd_read_packets,
            exited_flag.clone(),
            rcv_exiting,
        ));

        Self {
            name,
            snd_write_packets,
            rcv_read_packets,
            exited_flag,
            exiting_flag: AtomicBool::new(false),
            snd_exiting,
        }
    }

    pub fn status(&self) -> SocketStatus {
        if self.exited_flag.load(std::sync::atomic::Ordering::SeqCst) {
            SocketStatus::Closed
        } else if self.exiting_flag.load(std::sync::atomic::Ordering::SeqCst) {
            SocketStatus::Closing
        } else {
            SocketStatus::Running
        }
    }

    pub fn close(&self) {
        self.exiting_flag.store(self.snd_exiting.send(()).is_err(), std::sync::atomic::Ordering::SeqCst);
    }

    pub fn receive(&mut self, num_packets: Option<usize>) -> io::Result<Vec<P>> {
        let current_status = self.status();
        if !current_status.is_running() {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                format!(
                    "{} received failed due to closed or closing read connection, status={current_status:?}",
                    self.name
                ),
            ));
        }
        let mut packets = if let Some(n) = num_packets {
            if n == 0 {
                return Ok(vec![]);
            }
            Vec::with_capacity(n)
        } else {
            Vec::new()
        };
        while let Ok(p) = self.rcv_read_packets.try_recv() {
            // should be okay to ignore TryRecvError::Disconnected first, b/c the next receive call
            // should trigger the above conditional.
            packets.push(p);
            match num_packets {
                Some(n) if packets.len() >= n => break,
                _ => {},
            };
        }
        Ok(packets)
    }

    pub fn write(&self, b: impl Into<bytes::Bytes>) -> io::Result<()> {
        let current_status = self.status();
        if !current_status.is_running() {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                format!(
                    "{} send write failed due to socket shutting down or has already shut down: status={current_status:?}",
                    self.name
                ),
            ));
        }
        if let Err(e) = self.snd_write_packets.send(b.into()) {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                format!("{} send write failed due to already closed receiver, err={e}", self.name),
            ));
        }
        Ok(())
    }

    /// Replaces GetRemoteIpAddress in TC/AC
    pub fn remote_name(&self) -> &AddressOrName {
        &self.name
    }
}

#[instrument(skip_all, fields(target="network", name=%name))]
async fn handle_socket<R, W, P>(
    name: String,
    mut rd: R,
    mut wr: W,
    mut rcv_write_packets: mpsc::UnboundedReceiver<bytes::Bytes>,
    snd_read_packets: mpsc::UnboundedSender<P>,
    exited_flag: Arc<AtomicBool>,
    mut rcv_exiting: mpsc::UnboundedReceiver<()>,
) where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
    P: DecodeValueFromBytes,
{
    let mut write_flush_interval = tokio::time::interval(Duration::from_secs(1));
    write_flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut read_buffer_interval = tokio::time::interval(Duration::from_secs(10));
    read_buffer_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut read_buffer = MessageBuffer::default();

    loop {
        tokio::select! {
            res = rcv_exiting.recv() => {
                debug!(cause=?res, "received the signal to begin exiting, breaking");
                break;
            },
            _ = write_flush_interval.tick() => match wr.flush().await {
                Err(e) if e.kind() != io::ErrorKind::WriteZero => {
                    error!(cause=?e, "shutdown as encountered non recoverable error when flushing buffers");
                    break;
                },
                _ => continue,
            },
            _ = read_buffer_interval.tick() => {
                read_buffer.normalise();
            },
            read_res = rd.read_buf(&mut *read_buffer) => {
                match read_res {
                    Err(e) => {
                        error!(cause=%e, "shutdown read socket due to error in reading from stream");
                        break;
                    },
                    Ok(_) => {
                        let packet = match P::decode_from_bytes(&mut read_buffer) {
                            Err(BufferDecodeError::InsufficientBytes{have, wanted}) => {
                                warn!("insufficient bytes, continuing to read more, have {have} but wanted {wanted}");
                                continue;
                            },
                            Err(e) => {
                                error!(cause=%e, "error decoding data, possible socket corrruption? terminating");
                                break;
                            },
                            Ok(p) => p,
                        };
                        if snd_read_packets.send(packet).is_err() {
                            error!("unable to send read packets back, breaking out of handle socket loop");
                            break;
                        }
                    },
                }
            },
            opt_by = rcv_write_packets.recv() => {
                match opt_by {
                    None => {
                        error!("shutdown write socket to due to write receiver channel closing");
                        break;
                    },
                    Some(mut b) => {
                        if let Err(e) = wr.write_all_buf(&mut b).await {
                            error!(cause=?e, "shutdown write socket to due to write receiver channel closing");
                            break;
                        }
                    },
                }
            },
        };
    }
    rcv_exiting.close();
    // Shutdown everything, cleaning up
    if let Err(e) = wr.shutdown().await {
        error!(cause=?e, "shutdown write socket error: {e}");
    }
    exited_flag.store(true, std::sync::atomic::Ordering::SeqCst);
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
