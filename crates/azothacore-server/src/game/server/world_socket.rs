use std::{
    io,
    time::{Duration, Instant},
};

use azothacore_common::{
    AccountTypes,
    AzError,
    AzResult,
    bevy_app::{TokioRuntime, az_startup_succeeded},
    chrono::{DateTime, TimeDelta, Utc},
    configuration::ConfigMgr,
    hex_str,
    utils::{hex_str_to_bytes, unix_now},
};
use azothacore_database::{
    DbRow,
    args,
    args_unwrap,
    database_env::{LoginDatabase, LoginPreparedStmts},
};
use bevy::prelude::*;
use bnet_rpc::BattlenetRpcErrorCode;
use hmac::{Hmac, KeyInit, Mac};
use num::FromPrimitive;
use rand::{RngCore, TryRngCore, rngs::OsRng};
use sha2::{Digest, Sha256};
use sqlx::Row;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, tcp::OwnedWriteHalf},
    sync::{
        OwnedSemaphorePermit,
        mpsc::{UnboundedReceiver, UnboundedSender, error::TryRecvError, unbounded_channel},
    },
};
use tracing::{debug, error, trace};
use wow_db2::Locale;

use crate::{
    game::{
        globals::object_mgr::ClassExpansionRequirementStore,
        scripting::script_mgr::ScriptMgr,
        server::{
            packet::{ClientWorldPacket, ServerWorldPacket},
            packets::authentication::{
                AuthChallengeServerPacket,
                AuthResponseServerPacket,
                AuthSessionClientPacket,
                AuthSuccessInfo,
                BillingInfo,
                EnableEncryptionServerPacket,
                PingClientPacket,
                PongServerPacket,
                VirtualRealmInfo,
                VirtualRealmNameInfo,
            },
            protocol::opcodes::{CMSG, ConnectionType, OpcodeClient, OpcodeServer},
            world_session::WorldSession,
        },
        world::{CurrentRealm, WorldConfig},
    },
    shared::{
        crypto::{SessionKeyGenerator, WorldPacketCrypt},
        networking::{
            socket::AddressOrName,
            socket_mgr::{ConnectionComponent, NewTcpConnection, SocketReceiver},
        },
        packets::{ByteBuffer, ByteBufferError},
        query_callbacks::QueryCallbacks,
        shared_defines::{CURRENT_EXPANSION, Expansion},
    },
};

/// enum that signals to WorldSocket what the to do, usually this corresponds to what
/// the TcpStream is sending
enum WorldSocketIn {
    /// Tells WorldSocket to run WorldSocket::HandleSendAuthSession
    SendAuthSession,
    Packet {
        cmd:     OpcodeClient,
        payload: bytes::Bytes,
    },
}

/// enum that signals to the WorldSocket::handle_tcp_stream what to send to the client's TcpStream
enum WorldSocketOut {
    /// set in HandleAuthSessionCallback in TC, just before EnableEncryptionServerPacket is sent.
    /// SessionKey is used as a seed to auth_crypto on CMSG::EnableEncryptionAck.
    SetSessionKey([u8; 40]),
    SetConnectionType(ConnectionType),
    SetEncryptDecryptSeeds(([u8; 16], [u8; 16])),
    Packet {
        opcode:  OpcodeServer,
        payload: bytes::Bytes,
    },
    /// DelayedCloseSocket: flush whatever is queued, then shut down.
    DelayedClose,
}

#[derive(Component)]
/// PendingWorldSession represents the newly created WorldSession right after
/// HandleAuthSessionCallback is called.
/// This world session is not valid yet until we sent
/// SMSG::EnableEncryption + subsequently receive CMSG::HandleEnableEncryptionAck
/// From our client.
pub struct PendingWorldSession(WorldSession);

#[derive(Component)]
/// This receives completed packets from the TCP connection. Also contains owned permit
/// which is effectively a counter for total number of TCP connections.
pub struct WorldSocketReceiver {
    in_receiver: UnboundedReceiver<WorldSocketIn>,
    _permit:     Option<OwnedSemaphorePermit>,
}

impl Default for WorldSocketReceiver {
    fn default() -> Self {
        let (_, in_receiver) = unbounded_channel();
        Self { in_receiver, _permit: None }
    }
}

#[derive(Component, Clone)]
#[require(WorldSocketReceiver)]
pub struct WorldSocket {
    r#type:           ConnectionType,
    out_sender:       UnboundedSender<WorldSocketOut>, // i.e. This sends out to the Client
    remote_address:   AddressOrName,
    ip_country:       Option<String>,
    last_ping_time:   Option<Instant>,
    over_speed_pings: u32,
    server_challenge: [u8; 16],
}

#[derive(Component)]
/// Takes the place of WorldSocket::_authed property in TC / AC
/// The presence of this denotes that _authed == true
pub struct WorldSocketAuthed;

#[derive(Debug, thiserror::Error)]
pub enum ConnHandlerError {
    #[error("CloseSocket")]
    CloseSocket,
    #[error("ContinueReading")]
    ContinueReading,
}

impl From<ByteBufferError> for ConnHandlerError {
    fn from(value: ByteBufferError) -> Self {
        error!(cause=?value, "error reading packet, converting to ConnHandlerError::CloseSocket");
        ConnHandlerError::CloseSocket
    }
}

impl From<sqlx::Error> for ConnHandlerError {
    fn from(value: sqlx::Error) -> Self {
        error!(cause=?value, "sqlx::error in handling packets, converting to ConnHandlerError::CloseSOcket");
        ConnHandlerError::CloseSocket
    }
}

#[derive(sqlx::FromRow)]
struct IpInfoRow {
    banned:  u32,
    country: Option<String>,
}

struct AccountInfo {
    game_id:                    u32,
    game_key_data:              [u8; 64],
    game_expansion:             Expansion,
    game_mute_time:             i64,
    game_recruiter:             u32,
    game_os:                    String,
    game_is_recruiter:          bool,
    game_security:              AccountTypes,
    game_is_banned:             bool,
    battle_net_id:              u32,
    battle_net_is_locked_to_ip: bool,
    battle_net_last_ip:         String,
    battle_net_lock_country:    String,
    battle_net_locale:          Locale,
    battle_net_is_banned:       bool,
}

impl AccountInfo {
    fn is_banned(&self) -> bool {
        self.battle_net_is_banned || self.game_is_banned
    }
}

impl TryFrom<DbRow> for AccountInfo {
    type Error = AzError;

    fn try_from(fields: DbRow) -> AzResult<Self> {
        let game_id = fields.try_get(0)?;
        let key_data_hex: String = fields.try_get(1)?;
        let mut game_key_data = [0u8; _];
        hex_str_to_bytes(&key_data_hex, &mut game_key_data)?;
        let battle_net_last_ip = fields.try_get(2)?;
        let battle_net_is_locked_to_ip = fields.try_get(3)?;
        let battle_net_lock_country = fields.try_get(4)?;
        let game_expansion = Expansion::try_from(fields.get::<u8, _>(5) as i8).unwrap_or(CURRENT_EXPANSION);
        let game_mute_time = fields.try_get(6)?;
        let battle_net_locale = Locale::try_from(fields.get::<u32, _>(7)).unwrap_or(Locale::enUS);
        let game_recruiter = fields.try_get(8)?;
        let game_os = fields.try_get(9)?;
        let battle_net_id = fields.try_get(10)?;
        let game_security = fields
            .try_get::<u8, _>(11)
            .ok()
            .and_then(AccountTypes::from_u8)
            .unwrap_or(AccountTypes::SecConsole);
        let battle_net_is_banned = fields.try_get::<u32, _>(12).unwrap_or(0) != 0;
        let game_is_banned = fields.try_get::<u32, _>(13).unwrap_or(0) != 0;
        let game_is_recruiter = fields.try_get::<u32, _>(14).unwrap_or(0) != 0;

        Ok(Self {
            game_id,
            game_key_data,
            battle_net_last_ip,
            battle_net_is_locked_to_ip,
            battle_net_lock_country,
            game_expansion,
            game_mute_time,
            battle_net_locale,
            game_recruiter,
            game_os,
            battle_net_id,
            game_security,
            battle_net_is_banned,
            game_is_banned,
            game_is_recruiter,
        })
    }
}

/// WorldSocket::ServerConnectionInitialize
const SERVER_CONNECTION_INIT: &[u8] = b"WORLD OF WARCRAFT CONNECTION - SERVER TO CLIENT\n";
/// WorldSocket::ClientConnectionInitialize
const CLIENT_CONNECTION_INIT: &[u8] = b"WORLD OF WARCRAFT CONNECTION - CLIENT TO SERVER\n";

/// WorldSocket::AuthCheckSeed in TC
const WORLD_SOCKET_AUTH_CHECK_SEED: [u8; 16] = [0xC5, 0xC6, 0x98, 0x95, 0x76, 0x3F, 0x1D, 0xCD, 0xB6, 0xA1, 0x37, 0x28, 0xB3, 0x12, 0xFF, 0x8A];
/// WorldSocket::SessionKeySeed in TC
const WORLD_SOCKET_SESSION_KEY_SEED: [u8; 16] = [0x58, 0xCB, 0xCF, 0x40, 0xFE, 0x2E, 0xCE, 0xA6, 0x5A, 0x90, 0xB8, 0x01, 0x68, 0x6C, 0x28, 0x0B];
/// WorldSocket::ContinuedSessionSeed in TC
const WORLD_SOCKET_CONTINUED_SESSION_SEED: [u8; 16] = [0x16, 0xAD, 0x0C, 0xD4, 0x46, 0xF9, 0x4F, 0xB2, 0xEF, 0x7D, 0xEA, 0x2A, 0x17, 0x66, 0x4D, 0x2F];

/// ClientTypeSeed_Win in TC
const CLIENT_TYPE_SEED_WIN: [u8; 16] = [0x79, 0x7E, 0xCC, 0x19, 0x66, 0x2D, 0xCB, 0xD5, 0x09, 0x0A, 0x44, 0x81, 0x17, 0x3F, 0x1D, 0x26];
/// ClientTypeSeed_Wn64 in TC
const CLIENT_TYPE_SEED_WN64: [u8; 16] = [0x6E, 0x21, 0x2D, 0xEF, 0x6A, 0x01, 0x24, 0xA3, 0xD9, 0xAD, 0x07, 0xF5, 0xE3, 0x22, 0xF7, 0xAE];
/// ClientTypeSeed_Mc64 in TC
const CLIENT_TYPE_SEED_MC64: [u8; 16] = [0x34, 0x1C, 0xFE, 0xFE, 0x3D, 0x72, 0xAC, 0xA9, 0xA4, 0x40, 0x7D, 0xC5, 0x35, 0xDE, 0xD6, 0x6A];

impl WorldSocket {
    fn new(out_sender: UnboundedSender<WorldSocketOut>, remote_address: AddressOrName, ip_country: Option<String>) -> Self {
        let mut server_challenge = [0u8; 16];
        OsRng.unwrap_err().fill_bytes(&mut server_challenge);

        Self {
            r#type: ConnectionType::Realm,
            out_sender,
            remote_address,
            ip_country,
            server_challenge,
            last_ping_time: None,
            over_speed_pings: 0,
        }
    }

    /// Replaces GetRemoteIpAddress in TC/AC
    fn remote_ip_address(&self) -> String {
        self.remote_address.ip_str_or_name()
    }

    /// NOTE: equivalent to WorldSocket::Start in AC / TC
    fn start(login_db: Res<LoginDatabase>, callbacks: Res<QueryCallbacks>, mut sock_recv: ResMut<SocketReceiver<WorldSocket>>) {
        let login_db = login_db.clone();
        while let Ok(NewTcpConnection { permit, name, conn }) = sock_recv.0.try_recv() {
            let login_db = login_db.clone();
            let ip = name.ip_str_or_name();
            callbacks.add_commands_system_callback(
                async move {
                    (
                        LoginDatabase::sel_ip_info::<_, IpInfoRow>(&*login_db, args_unwrap!(ip)).await,
                        conn,
                        permit,
                        name,
                    )
                },
                WorldSocket::check_ip_callback,
            );
        }
    }

    /// WorldSocket::SendPacket / WorldSocket::SendPacketAndLogOpcode in TC / AC
    pub fn send_packet<P: ServerWorldPacket>(&self, packet: P) {
        let opcode = P::OPCODE;
        trace!(target:"network.opcode", remote=self.remote_ip_address(), ?opcode, "S->C"); // SendPacketAndLogOpcode

        // TODO: Implement packet logging for easy debugging
        // if (sPacketLog->CanLogPacket()) {
        //     sPacketLog->LogPacket(packet, SERVER_TO_CLIENT, GetRemoteIpAddress(), GetRemotePort(), GetConnectionType());
        // }

        _ = self.out_sender.send(WorldSocketOut::Packet {
            opcode,
            payload: packet.write().into(),
        });
    }

    /// WorldSocket::SendAuthResponseError in TC / AC
    /// Marks the socket for closing after write buffer becomes empty
    pub fn send_auth_response_error(&self, code: BattlenetRpcErrorCode) {
        self.send_packet(AuthResponseServerPacket {
            result:       code as u32,
            success_info: None,
            wait_info:    None,
        });
    }

    /// WorldSocket::DelayedCloseSocket in TC / AC
    pub fn delayed_close(&self) {
        _ = self.out_sender.send(WorldSocketOut::DelayedClose);
    }

    /// WorldSocket::ReadHeaderHandler in TC
    fn read_header_handler(
        auth_crypt: &mut Option<WorldPacketCrypt>,
        rbuf: &mut bytes::BytesMut,
        header: &mut Option<(usize, OpcodeClient)>,
    ) -> Result<(usize, OpcodeClient), ConnHandlerError> {
        match header {
            // header already decrypted but buffer doesn't have enough data
            Some((size, cmd)) if rbuf.len() < *size => return Err(ConnHandlerError::ContinueReading),
            Some((size, cmd)) => {
                // Header is read, and buffer is ready to accept a payload
                let res = Ok((*size, *cmd));
                *header = None;
                return res;
            },
            None => {},
        }

        if rbuf.len() < 6 {
            // Couldn't receive the whole header this time.
            return Err(ConnHandlerError::ContinueReading);
        }
        // ReadHeaderHandler routine
        let mut h = rbuf.split_to(6);
        if let Some(c) = auth_crypt {
            c.decrypt_recv(&mut h[..4]);
        }
        let size = u32::from_le_bytes(h[..4].try_into().unwrap()) as usize - 2;
        let cmd = u16::from_le_bytes(h[4..6].try_into().unwrap());
        if size >= 0x10000 {
            error!(target:"network", size, cmd, "client sent malformed packet, too big");
            return Err(ConnHandlerError::CloseSocket);
        }
        let Some(cmd) = OpcodeClient::from_u16(cmd) else {
            error!(target:"network", size, cmd, "client sent malformed packet, opcode not expected");
            return Err(ConnHandlerError::CloseSocket);
        };

        if rbuf.len() < size {
            // header already decrypted but buffer doesn't have enough data
            // save the header.
            *header = Some((size, cmd));
            return Err(ConnHandlerError::ContinueReading);
        }
        // Header is read, and buffer is ready to accept a payload
        *header = None;
        Ok((size, cmd))
    }

    /// Socket::AsyncRead => Socket::ReadHandlerInternal => WorldSocket::ReadHandler routine in TC.
    #[expect(clippy::too_many_arguments)]
    fn read_handler(
        res: Result<usize, io::Error>,
        auth_crypt: &mut Option<WorldPacketCrypt>,
        rbuf: &mut bytes::BytesMut,
        header: &mut Option<(usize, OpcodeClient)>,
        session_key: &mut Option<[u8; 40]>,
        conn_type: &ConnectionType,
        encrypt_seed: &[u8; 16],
        decrypt_seed: &[u8; 16],
        in_snd: &UnboundedSender<WorldSocketIn>,
    ) -> ConnHandlerError {
        match res {
            // If no bytes are read, assume that its a TCP shutdown and close socket.
            Ok(0) => return ConnHandlerError::CloseSocket,
            Err(e) => {
                trace!(cause=?e, "closing socket due to read error");
                return ConnHandlerError::CloseSocket;
            },
            _ => {},
        }

        // ReadHandler: decode EVERY complete packet in the buffer, not just one
        loop {
            let (size, cmd) = match Self::read_header_handler(auth_crypt, rbuf, header) {
                Err(e) => return e,
                Ok(v) => v,
            };
            let payload = rbuf.split_to(size).freeze();

            if cmd == CMSG::EnableEncryptionAck
                && auth_crypt.is_none()
                && let Some(k) = session_key.take()
            {
                // HandleEnableEncryptionAck: start the crypto BEFORE the next header is read.
                // NOTE:
                // 1. the session_key is taken from after the `EnableEncryptionServerPacket` is called
                // packets are expected to come in order so this is usually not an issue.
                // 2. encrypt and decrypt seeds are taken from right after we send `AuthChallengeServerPacket`
                *auth_crypt = Some(match *conn_type {
                    ConnectionType::Realm => WorldPacketCrypt::new(&k),
                    ConnectionType::Instance => WorldPacketCrypt::new_with_keys(&k, encrypt_seed, decrypt_seed),
                });
            }

            // Sends the packet to WorldSocket::ReadDataHandler
            if in_snd.send(WorldSocketIn::Packet { cmd, payload }).is_err() {
                return ConnHandlerError::CloseSocket;
            }
        }
    }

    /// WorldSocket::Update and WorldSocket::WritePacketToBuffer logic in TC
    /// TODO: Include grouping of packets, compression & size estimates
    async fn write_handler(
        msg: Option<WorldSocketOut>,
        wr: &mut OwnedWriteHalf,
        auth_crypt: &mut Option<WorldPacketCrypt>,
        session_key: &mut Option<[u8; 40]>,
        conn_type: &mut ConnectionType,
        encrypt_seed: &mut [u8; 16],
        decrypt_seed: &mut [u8; 16],
    ) -> ConnHandlerError {
        let (opcode, payload) = match msg {
            None => return ConnHandlerError::CloseSocket,
            Some(WorldSocketOut::SetSessionKey(k)) => {
                *session_key = Some(k);
                return ConnHandlerError::ContinueReading;
            },
            Some(WorldSocketOut::DelayedClose) => {
                _ = wr.flush().await;
                return ConnHandlerError::CloseSocket;
            },
            Some(WorldSocketOut::SetConnectionType(typ)) => {
                *conn_type = typ;
                return ConnHandlerError::ContinueReading;
            },
            Some(WorldSocketOut::SetEncryptDecryptSeeds((e, d))) => {
                *encrypt_seed = e;
                *decrypt_seed = d;
                return ConnHandlerError::ContinueReading;
            },
            Some(WorldSocketOut::Packet { opcode, payload }) => (opcode, payload),
        };

        // WritePacketToBuffer; compression above 0x400 bytes is skipped for now
        let opcode = opcode as u16;
        let opcode_le_bytes = opcode.to_le_bytes();
        let packet_size = (payload.len() + opcode_le_bytes.len()) as u32;

        let mut header = [0u8; 6];
        header[..4].copy_from_slice(&packet_size.to_le_bytes());
        header[4..].copy_from_slice(&opcode_le_bytes);

        if let Some(c) = auth_crypt {
            c.encrypt_send(&mut header[..4]);
        }
        if wr.write_all(&header).await.is_err() || wr.write_all(&payload).await.is_err() {
            return ConnHandlerError::CloseSocket;
        }

        ConnHandlerError::ContinueReading
    }

    /// Handles the underlying TCP connection to the user.
    /// Use `in_snd` to influence the underlying WorldSocket. Usually used in response to new packets from
    /// the underlying TCP connection, but also used right after the initial server/client init handshake via
    /// SERVER_CONNECTION_INIT / CLIENT_CONNECTION_INIT.
    /// Take in from `out_rcv` to write packets to `conn`.
    async fn handle_tcp_stream(mut conn: TcpStream, mut out_rcv: UnboundedReceiver<WorldSocketOut>, in_snd: UnboundedSender<WorldSocketIn>) {
        // NOTE: Tail end of WorldSocket::CheckIpCallback  + WorldSocket::InitializeHandler
        // writes out to conn, this is triggered before any new out_rcv message as a handshake
        // to client is needed as per 7.3.5 protocol.
        if conn.write_all(SERVER_CONNECTION_INIT).await.is_err() {
            return;
        }
        let mut init = [0u8; CLIENT_CONNECTION_INIT.len()];
        if let Err(e) = conn.read_exact(&mut init).await {
            trace!(target:"network", cause=?e, "error reading client connection handshake response");
            return;
        }
        if init != CLIENT_CONNECTION_INIT {
            trace!(target:"network", "init string does not match, got '{}'", String::from_utf8_lossy(&init));
            return;
        }
        // TODO: Setup package compression
        // _compressionStream = new z_stream();
        // _compressionStream->zalloc = (alloc_func)NULL;
        // _compressionStream->zfree = (free_func)NULL;
        // _compressionStream->opaque = (voidpf)NULL;
        // _compressionStream->avail_in = 0;
        // _compressionStream->next_in = NULL;
        // int32 z_res = deflateInit2(_compressionStream, sWorld->getIntConfig(CONFIG_COMPRESSION), Z_DEFLATED, -15, 8, Z_DEFAULT_STRATEGY);
        // if (z_res != Z_OK)
        // {
        //     CloseSocket();
        //     TC_LOG_ERROR("network", "Can't initialize packet compression (zlib: deflateInit) Error code: %i (%s)", z_res, zError(z_res));
        //     return;
        // }
        if let Err(e) = in_snd.send(WorldSocketIn::SendAuthSession) {
            trace!(target:"network", cause=?e,"error sending to WorldSocket to start HandleSendAuthSession");
            return;
        }

        let (mut rd, mut wr) = conn.into_split();
        let mut session_key: Option<[u8; 40]> = None;
        let mut auth_crypt = None;
        let mut connection_type = ConnectionType::Realm;
        let mut encrypt_seed = [0u8; 16];
        let mut decrypt_seed = [0u8; 16];
        let mut rbuf = bytes::BytesMut::with_capacity(4096);
        let mut client_header: Option<(usize, OpcodeClient)> = None; // header already decrypted, waiting for the payload

        loop {
            let res = tokio::select! {
                read_res = rd.read_buf(&mut rbuf) => Self::read_handler(read_res, &mut auth_crypt, &mut rbuf, &mut client_header, &mut session_key, &connection_type, &encrypt_seed, & decrypt_seed, &in_snd),
                msg = out_rcv.recv() => Self::write_handler(msg, &mut wr, &mut auth_crypt, &mut session_key, &mut connection_type, &mut encrypt_seed, &mut decrypt_seed).await,
            };
            if matches!(res, ConnHandlerError::CloseSocket) {
                break;
            }
        }
        _ = wr.shutdown().await;
    }

    /// NOTE: equivalent to WorldSocket::CheckIpCallback in AC / TC
    #[expect(clippy::type_complexity)]
    fn check_ip_callback(
        In((res, conn, _permit, name)): In<(Result<Vec<IpInfoRow>, sqlx::Error>, TcpStream, Option<OwnedSemaphorePermit>, AddressOrName)>,
        mut commands: Commands,
        rt: Res<TokioRuntime>,
    ) {
        let res = match res {
            Err(e) => {
                error!(target: "network", cause=?e, "WorldSocket::CheckIpCallback DB access error");
                // DelayedCloseSocket => nothing spawned yet, so return just drops the TcpStream.
                return;
            },
            Ok(v) => v,
        };

        let is_banned = res.iter().any(|fields| fields.banned != 0);
        if is_banned {
            error!(target:"network", "WorldSocket::CheckIpCallback: Sent Auth Response (IP {name} banned).");
            // DelayedCloseSocket => nothing spawned yet, so return just drops the TcpStream.
            return;
        }
        let ip_country = res.into_iter().find_map(|v| v.country);
        let (out_snd, out_rcv) = unbounded_channel();
        let (in_snd, in_rcv) = unbounded_channel();

        rt.spawn(Self::handle_tcp_stream(conn, out_rcv, in_snd));

        commands.spawn((Self::new(out_snd, name, ip_country), WorldSocketReceiver { in_receiver: in_rcv, _permit }));
    }

    /// WorldSocket::HandleSendAuthSession in TC
    fn handle_send_auth_session(&mut self) {
        let mut encrypt_seed = [0u8; 16];
        let mut decrypt_seed = [0u8; 16];
        OsRng.unwrap_err().fill_bytes(&mut encrypt_seed);
        OsRng.unwrap_err().fill_bytes(&mut decrypt_seed);

        let mut dos_challenge = [0u8; 32];
        dos_challenge[..16].copy_from_slice(&encrypt_seed);
        dos_challenge[16..].copy_from_slice(&decrypt_seed);

        _ = self.out_sender.send(WorldSocketOut::SetEncryptDecryptSeeds((encrypt_seed, decrypt_seed)));
        self.send_packet(AuthChallengeServerPacket {
            challenge: self.server_challenge,
            dos_challenge,
            dos_zero_bits: 1,
        });
    }

    /// WorldSocket::HandlePing
    fn handle_ping(&mut self, cfg: &WorldConfig, world_session: Option<&WorldSession>, ping: PingClientPacket) -> bool {
        let now = Instant::now();
        if let Some(last) = self.last_ping_time.replace(now) {
            let diff = now - last;
            if diff < Duration::from_secs(27) {
                self.over_speed_pings += 1;

                match cfg.MaxOverspeedPings {
                    Some(max_allowed) if (self.over_speed_pings > max_allowed) && world_session.is_some() => {
                        // TODO: RBAC_PERM_SKIP_CHECK_OVERSPEED_PING check: AZC-PARTIAL until RBAC (M2)
                        let player_info: Option<String> = None; // TODO: Implement WorldSession::GetPlayerInfo
                        error!(target:"network", player_info, remote=self.remote_ip_address(),  "WorldSocket::HandlePing: kicked for over-speed pings");
                        // _worldSession->GetPlayerInfo().c_str(),
                        // GetRemoteIpAddress().to_string().c_str(),
                        return false;
                    },
                    _ => {},
                }
            } else {
                self.over_speed_pings = 0;
            }
        }

        let Some(sess) = world_session else {
            error!(target:"network", "WorldSocket::HandlePing: peer sent CMSG_PING, but is not authenticated or got recently kicked, address = {:?}", self.remote_ip_address());
            return false;
        };

        sess.set_ping(ping.latency);
        self.send_packet(PongServerPacket { serial: ping.serial });
        true
    }

    /// WorldSocket::HandleAuthSession in TC
    fn handle_auth_session(
        &mut self,
        login_db: &LoginDatabase,
        callbacks: &QueryCallbacks,
        realm: &CurrentRealm,
        entity: Entity,
        auth_session: AuthSessionClientPacket,
    ) -> Result<(), ConnHandlerError> {
        // Get the account information from the auth database
        let login_db = (*login_db).clone();
        let query_args = args!(realm.id.realm, &auth_session.realm_join_ticket)?;

        self.r#type = ConnectionType::Realm;
        _ = self.out_sender.send(WorldSocketOut::SetConnectionType(self.r#type.clone()));

        callbacks.add_commands_entity_system_callback(
            entity,
            async move {
                let res = LoginDatabase::sel_account_info_by_name(&*login_db, query_args).await;
                (res, auth_session)
            },
            Self::handle_auth_session_callback,
        );

        Ok(())
    }

    /// WorldSocket::HandleAuthSessionCallback in TC / AC
    #[expect(clippy::too_many_arguments, clippy::type_complexity)]
    fn handle_auth_session_callback(
        In((entity, (res, auth_session))): In<(Entity, (Result<Option<DbRow>, sqlx::Error>, AuthSessionClientPacket))>,
        mut commands: Commands,
        rt: Res<TokioRuntime>,
        login_db: Res<LoginDatabase>,
        cfg: Res<ConfigMgr<WorldConfig>>,
        realm: Res<CurrentRealm>,
        sock_query: Query<&Self>,
    ) {
        let Ok(this) = sock_query.get(entity) else {
            return;
        };

        macro_rules! return_delayed_close {
            () => {{
                _ = this.delayed_close();
                return;
            }};
        }
        macro_rules! handled_db_args {
            ( $($arg:expr),* ) => {
                match args!($($arg),*) {
                    Err(e) => {
                        error!(target:"network", cause=?e, "WorldSocket::HandleAuthSession: DB args error");
                        return_delayed_close!();
                    },
                    Ok(v) => v,
                }
            }
        }

        // Stop if the account is not found
        let row = match res {
            Ok(Some(r)) => r,
            Ok(None) => {
                // We can not log here, as we do not know the account. Thus, no accountId.
                error!(target:"network", "WorldSocket::HandleAuthSession: Sent Auth Response (unknown account).");
                return_delayed_close!();
            },
            Err(e) => {
                error!(target:"network", cause=?e, "WorldSocket::HandleAuthSession: DB error");
                return_delayed_close!();
            },
        };
        let account = match AccountInfo::try_from(row) {
            Ok(v) => v,
            Err(e) => {
                error!(target:"network", cause=?e, "cannot decode account info from SQL result");
                return_delayed_close!();
            },
        };

        // For hook purposes, we get Remoteaddress at this point.
        let address = if cfg.AllowLoggingIPAddressesInDatabase {
            this.remote_ip_address()
        } else {
            String::from("0.0.0.0")
        };
        let address = address.as_str();

        // As we don't know if attempted login process by ip works, we update last_attempt_ip right away
        let login_db_cloned = login_db.clone();
        let args = handled_db_args!(address, &auth_session.realm_join_ticket);
        rt.spawn(async move { LoginDatabase::upd_last_attempt_ip(&*login_db_cloned, args).await });
        // This also allows to check for possible "hack" attempts on account

        let mut digest_key_hash = Sha256::new().chain_update(account.game_key_data);
        match account.game_os.as_str() {
            "Win" => digest_key_hash.update(CLIENT_TYPE_SEED_WIN),
            "Wn64" => digest_key_hash.update(CLIENT_TYPE_SEED_WN64),
            "Mc64" => digest_key_hash.update(CLIENT_TYPE_SEED_MC64),
            _ => {},
        }

        let hmac = match Hmac::<Sha256>::new_from_slice(&digest_key_hash.finalize()) {
            Err(e) => {
                error!(target:"network", cause=?e, "WorldSocket::HandleAuthSession: HMAC error");
                return_delayed_close!();
            },
            Ok(v) => v
                .chain_update(auth_session.local_challenge)
                .chain_update(this.server_challenge)
                .chain_update(WORLD_SOCKET_AUTH_CHECK_SEED),
        };

        // Check that Key and account name are the same on client and server
        // NOTE: Verifies only the first 24 bytes, thus verify_truncate_left
        if let Err(e) = hmac.verify_truncated_left(&auth_session.digest) {
            error!(target:"network", cause=?e, "Authentication failed for account: {} ('{}') address: {address:?}", account.game_id, auth_session.realm_join_ticket);
            return_delayed_close!();
        }

        let key_data = Sha256::new().chain_update(account.game_key_data).finalize();
        let session_key_hmac = match Hmac::<Sha256>::new_from_slice(&key_data) {
            Err(e) => {
                error!(target:"network", cause=?e, "WorldSocket::HandleAuthSession: HMAC error");
                return_delayed_close!();
            },
            Ok(v) => v
                .chain_update(this.server_challenge)
                .chain_update(auth_session.local_challenge)
                .chain_update(WORLD_SOCKET_SESSION_KEY_SEED)
                .finalize(),
        };

        let mut session_key = [0u8; _];
        SessionKeyGenerator::<Sha256>::new(&session_key_hmac.into_bytes()).generate(&mut session_key);

        let login_db_cloned = login_db.clone();
        let args = handled_db_args!(hex_str!(session_key), account.game_id);
        rt.spawn(async move { LoginDatabase::upd_account_info_continued_session(&*login_db_cloned, args).await });

        // // First reject the connection if packet contains invalid data or realm state doesn't allow logging in
        // if (sWorld->IsClosed())
        // {
        //     SendAuthResponseError(ERROR_DENIED);
        //     TC_LOG_ERROR("network", "WorldSocket::HandleAuthSession: World closed, denying client (%s).", GetRemoteIpAddress().to_string().c_str());
        //     DelayedCloseSocket();
        //     return;
        // }

        if auth_session.realm_id != realm.id.realm {
            this.send_auth_response_error(BattlenetRpcErrorCode::Denied);
            error!(target:"network", address, requested_realm=auth_session.realm_id, current_realm=realm.id.realm, "WorldSocket::HandleAuthSession: Client requested connecting with another realm but this realm has a different id set in config.");
            return_delayed_close!();
        }

        // Must be done before WorldSession is created
        if cfg.Warden.Enabled && !matches!(account.game_os.as_str(), "Win" | "Wn64" | "Mc64") {
            this.send_auth_response_error(BattlenetRpcErrorCode::Denied);
            error!(target:"network", address, client_os=account.game_os, "WorldSocket::HandleAuthSession: Client attempted to log in using invalid client OS.");
            return_delayed_close!();
        }

        // // AZC-PARTIAL: azothacore-rs crates/azothacore-server/src/game/server/world_socket.rs
        // // TODO:  Implement proper _ipCountry handling, can possibly drop `ip_country` as well in WorldSocket.
        // // Sample impl for ip_country here, taken from AC
        // if (IpLocationRecord const* location = sIPLocation->GetLocationRecord(address))
        //     _ipCountry = location->CountryCode;

        //- Re-check ip locking (same check as in auth).
        if account.battle_net_is_locked_to_ip {
            if account.battle_net_last_ip != address {
                this.send_auth_response_error(BattlenetRpcErrorCode::RiskAccountLocked);
                debug!(
                    target:"network",
                    original_ip = account.battle_net_last_ip,
                    new_ip = address,
                    "WorldSocket::HandleAuthSession: Sent Auth Response (Account IP differs)."
                );
                // We could log on hook only instead of an additional db log, however action logger is config based. Better keep DB logging as well
                ScriptMgr::on_failed_account_login_deferred(&mut commands, account.game_id);
                return_delayed_close!();
            }
        } else if !matches!(account.battle_net_lock_country.as_str(), "" | "00")
            && this.ip_country.as_ref().is_some_and(|ipc| account.battle_net_lock_country != *ipc)
        {
            this.send_auth_response_error(BattlenetRpcErrorCode::RiskAccountLocked);
            debug!(
                target:"network",
                original_country = account.battle_net_lock_country,
                new_country = this.ip_country,
                "WorldSocket::HandleAuthSession: Sent Auth Response (Account country differs)."
            );
            ScriptMgr::on_failed_account_login_deferred(&mut commands, account.game_id);
            // We could log on hook only instead of an additional db log, however action logger is config based. Better keep DB logging as well
            return_delayed_close!();
        }

        // Negative mutetime indicates amount of seconds to be muted effective on next login - which is now.
        let mut update_mute = None;
        let mutetime = if account.game_mute_time > 0 {
            DateTime::from_timestamp_secs(account.game_mute_time)
        } else if account.game_mute_time == 0 {
            None
        } else {
            let dt = Utc::now() + TimeDelta::seconds(account.game_mute_time.abs());
            update_mute = Some(dt.timestamp());
            Some(dt)
        };
        if let Some(mutetime) = update_mute {
            let login_db_cloned = login_db.clone();
            let args = handled_db_args!(mutetime, account.game_id);
            rt.spawn(async move { LoginDatabase::upd_mute_time_login(&*login_db_cloned, args).await });
        }

        if account.is_banned() {
            this.send_auth_response_error(BattlenetRpcErrorCode::GameAccountBanned);
            error!(target:"network", "WorldSocket::HandleAuthSession: Sent Auth Response (Account banned).");
            ScriptMgr::on_failed_account_login_deferred(&mut commands, account.game_id);
            return_delayed_close!();
        }

        // Check locked state for server // TODO: IMPLEMENT sWorld->GetPlayerSecurityLimit
        let allowed_account_type = AccountTypes::SecPlayer; // sWorld->GetPlayerSecurityLimit();
        debug!(
            target:"network",
            allowed = ?allowed_account_type,
            player = ?account.game_security,
            "allowed account type level"
        );
        if allowed_account_type > AccountTypes::SecPlayer && account.game_security < allowed_account_type {
            this.send_auth_response_error(BattlenetRpcErrorCode::ServerIsPrivate);
            error!(
                target:"network",
                "WorldSocket::HandleAuthSession: User tries to login but his security level is not enough"
            );
            ScriptMgr::on_failed_account_login_deferred(&mut commands, account.game_id);
            return_delayed_close!();
        }

        debug!(
            target:"network",
            client = auth_session.realm_join_ticket,
            address = address,
            "WorldSocket::HandleAuthSession: Client authenticated successfully."
        );

        // Update the last_ip in the database as it was successful for login
        let args = handled_db_args!(address, &auth_session.realm_join_ticket);
        let login_db_clone = login_db.clone();
        rt.spawn(async move { LoginDatabase::upd_last_ip(&*login_db_clone, args).await });

        // At this point, we can safely hook a successful login
        ScriptMgr::on_account_login_deferred(&mut commands, account.game_id);
        ScriptMgr::on_last_ip_update_deferred(&mut commands, account.game_id, address.to_string());

        // Initialize Warden system only if it is enabled by config
        // TODO: Implement warden
        // if (wardenActive)
        //     _worldSession->InitWarden(&_sessionKey);

        commands.entity(entity).insert((
            WorldSocketAuthed,
            PendingWorldSession(WorldSession::new(
                account.game_id,
                auth_session.realm_join_ticket,
                account.battle_net_id,
                account.game_security,
                account.game_expansion.min(cfg.Expansion),
                account.game_expansion,
                mutetime,
                account.game_os,
                account.battle_net_locale,
                account.game_recruiter,
                account.game_is_recruiter,
                // TODO: WorldSession::ResetTimeOutTime: we set directly here
                // but I'd imagine this function will be moved to sWorld's equivalent
                // in AZC
                *cfg.SocketTimeOutTime,
            )),
        ));

        // // TODO: AC 3.3.5 takes addon info from auth_session, but that no longer exists
        // inside the packet for 7.3.5. From the below table, it appears that the TC code doesn't
        // handle it yet.
        // ┌────────────────────────────┬───────────────────────────────┬──────────────┬────────────┐
        // │           Opcode           │           TC 7.3.5            │ AshamaneCore │ LegionCore │
        // ├────────────────────────────┼───────────────────────────────┼──────────────┼────────────┤
        // │ CMSG_ADDON_LIST            │ STATUS_UNHANDLED, Handle_NULL │ same         │ same       │
        // ├────────────────────────────┼───────────────────────────────┼──────────────┼────────────┤
        // │ CMSG_REPORT_ENABLED_ADDONS │ STATUS_UNHANDLED, Handle_NULL │ same         │ same       │
        // ├────────────────────────────┼───────────────────────────────┼──────────────┼────────────┤
        // │ SMSG_REQUEST_ADDON_LIST    │ defined, never sent           │ same         │ same       │
        // └────────────────────────────┴───────────────────────────────┴──────────────┴────────────┘
        // _worldSession->ReadAddonsInfo(authSession->AddonInfo);

        // // TODO: AC 3.3.5 implements accountFlags taken from the SRP6 authserver
        // // that they set via the AUTH_LOGON_PROOF login packet in  the AccountFlags field
        // // There are a bunch of bnet account_service protobuf like GameAccountFlagUpdateRequest /
        // // GameAccountFlagUpdateRequest but they are not implemented in any TC cores.
        // _worldSession->ValidateAccountFlags();

        _ = this.out_sender.send(WorldSocketOut::SetSessionKey(session_key));
        this.send_packet(EnableEncryptionServerPacket);
    }

    /// WorldSocket::HandleEnableEncryptionAck in TC
    fn handle_enable_encryption_ack(&self, commands: &mut Commands, entity: Entity) {
        commands.entity(entity).queue(|mut ent: EntityWorldMut| {
            let Some(PendingWorldSession(s)) = ent.take() else {
                return;
            };
            // TODO: Implement the various sWorld->AddSession and sWorld->AddInstanceSocket methods
            //       For now just append the world session onto the same entity as world socket.
            // if (_type == CONNECTION_TYPE_REALM)
            // {
            //     _authCrypt.Init(&_sessionKey);
            //     sWorld->AddSession(_worldSession);
            // }
            // else
            // {
            //     _authCrypt.Init(&_sessionKey, _encryptSeed.AsByteArray().get(), _decryptSeed.AsByteArray().get());
            //     sWorld->AddInstanceSocket(shared_from_this(), _key);
            // }
            ent.insert(s);
        });
    }

    /// WorldSocket::ReadDataHandler in TC
    #[expect(clippy::too_many_arguments)]
    fn read_data_handler(
        &mut self,
        commands: &mut Commands,
        cfg: &WorldConfig,
        callbacks: &QueryCallbacks,
        login_db: &LoginDatabase,
        realm: &CurrentRealm,
        // Components params start
        entity: Entity,
        authed: Option<&WorldSocketAuthed>,
        world_session: Option<&mut WorldSession>,
        // Components params end
        opcode: OpcodeClient,
        mut packet_buffer: ByteBuffer,
    ) -> Result<(), ConnHandlerError> {
        // WorldSocket::LogOpcodeText is this line
        trace!(target:"network.opcode", remote=self.remote_ip_address(), ?opcode, "C->S");
        match opcode {
            PingClientPacket::OPCODE => {
                let ping = PingClientPacket::read(&mut packet_buffer)?;
                if !self.handle_ping(cfg, world_session.map(|v| &*v), ping) {
                    return Err(ConnHandlerError::CloseSocket);
                }
            },
            AuthSessionClientPacket::OPCODE => {
                if authed.is_some() {
                    error!("received duplicate {:?}", AuthSessionClientPacket::OPCODE);
                    return Err(ConnHandlerError::CloseSocket);
                }
                let auth_session = AuthSessionClientPacket::read(&mut packet_buffer)?;
                // WaitingForQuery: TC stops reading here. Our task keeps reading, but the client
                // sends nothing until it gets EnableEncryption, so this is safe.
                self.handle_auth_session(login_db, callbacks, realm, entity, auth_session)?;
            },
            // CMSG::AuthContinuedSession => {
            // TODO: CMSG_AUTH_CONTINUED_SESSION branch
            // },
            CMSG::KeepAlive | CMSG::LogDisconnect => {},
            CMSG::EnableNagle => { /* SetNoDelay(false): add SocketOut::SetNoDelay */ },
            // CMSG::ConnectToFailed => {
            // TODO: CMSG_CONNECT_TO_FAILED branch
            // },
            CMSG::EnableEncryptionAck => self.handle_enable_encryption_ack(commands, entity),
            op => {
                let Some(sess) = world_session else {
                    error!(target:"network.opcode", opcode=?op, "ProcessIncoming: Client not authed opcode");
                    return Err(ConnHandlerError::CloseSocket);
                };
                sess.set_timeout_time(*cfg.SocketTimeOutTime);
                // TODO: IMPL THE REST OF ReadDataHandler, this would be WorldSession::QueuePacket
                debug!(opcode = ?op, "Receive packet, dropping");
            },
        };

        Ok(())
    }

    #[expect(clippy::type_complexity)]
    fn read_world_sockets(
        mut commands: Commands,
        callbacks: Res<QueryCallbacks>,
        login_db: Res<LoginDatabase>,
        realm: Res<CurrentRealm>,
        cfg: Res<ConfigMgr<WorldConfig>>,
        mut socks: Query<(
            Entity,
            &mut Self,
            &mut WorldSocketReceiver,
            Option<&WorldSocketAuthed>,
            Option<&mut WorldSession>,
        )>,
    ) {
        for (e, mut sock, mut sock_rcv, authed, mut session) in &mut socks {
            loop {
                let (cmd, payload) = match sock_rcv.in_receiver.try_recv() {
                    Err(TryRecvError::Disconnected) => {
                        commands.entity(e).despawn(); // OnClose
                        break;
                    },
                    Err(TryRecvError::Empty) => break,
                    Ok(WorldSocketIn::SendAuthSession) => {
                        sock.handle_send_auth_session();
                        continue;
                    },
                    Ok(WorldSocketIn::Packet { cmd, payload }) => (cmd, payload),
                };
                if let Err(ConnHandlerError::CloseSocket) = sock.read_data_handler(
                    &mut commands,
                    &cfg,
                    &callbacks,
                    &login_db,
                    &realm,
                    e,
                    authed,
                    session.as_deref_mut(),
                    cmd,
                    payload.into(),
                ) {
                    _ = sock.out_sender.send(WorldSocketOut::DelayedClose);
                    commands.entity(e).despawn();
                    break;
                }
            }
        }
    }
}

impl ConnectionComponent for WorldSocket {}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddWorldSocketSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldSocketReadPacketsSet;

pub fn world_socket_handling_plugin(app: &mut App) {
    app.add_systems(FixedUpdate, WorldSocket::start.run_if(az_startup_succeeded()).in_set(AddWorldSocketSet))
        .add_systems(
            Update,
            WorldSocket::read_world_sockets.run_if(az_startup_succeeded()).in_set(WorldSocketReadPacketsSet),
        )
        // TODO: Temp system, replace me
        .add_systems(Update, send_auth_response_on_new_session);
}

/// TODO: Temporary M1 system, AZC-PARTIAL on AddSession_/InitializeSession:
/// for each newly added WorldSession, SendAuthResponse(ERROR_OK, false).
/// M2 replaces this with InitializeSession's account data / tutorials query chain.
fn send_auth_response_on_new_session(
    q: Query<(&WorldSocket, &WorldSession), Added<WorldSession>>,
    realm: Res<CurrentRealm>,
    classes: Res<ClassExpansionRequirementStore>,
) {
    for (sock, s) in &q {
        let addr = realm.id.get_address();
        sock.send_packet(AuthResponseServerPacket {
            result:       BattlenetRpcErrorCode::Ok as u32,
            success_info: Some(AuthSuccessInfo {
                account_expansion_level: s.account_expansion as u8,
                active_expansion_level: s.expansion as u8, // min(account expansion, cfg.Expansion), as in WorldSession ctor
                virtual_realm_address: addr,
                time: unix_now().as_secs() as i32,
                virtual_realms: vec![VirtualRealmInfo {
                    realm_name_info: VirtualRealmNameInfo {
                        is_local:              true,
                        is_internal_realm:     false,
                        realm_name_actual:     realm.name.clone(),
                        realm_name_normalised: realm.normalised_name.clone(),
                    },
                    realm_address:   addr,
                }],
                available_classes: classes.clone(),
                billing: BillingInfo {
                    billing_plan: 0,
                    time_remain:  0,
                    in_game_room: false,
                    unknown735:   0,
                },
                currency_id: 0,
                time_rested: 0,
                force_character_template: false,
                is_expansion_trial: false,
                num_players_alliance: None,
                num_players_horde: None,
                time_seconds_until_pc_kick: 0,
            }),
            wait_info:    None,
        });
    }
}
