use std::{
    collections::BTreeMap,
    io::{self, Write},
    ops::{Deref, DerefMut},
    process,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
        OnceLock,
    },
};

use azothacore_common::{
    az_error,
    bevy_app::{az_startup_succeeded, TokioRuntime},
    configuration::ConfigMgr,
    deref_boilerplate,
    utils::{unix_now, BufferDecodeError, BufferResult, DecodeValueFromBytes, MessageBuffer},
    AccountTypes,
    AzResult,
    Locale,
};
use azothacore_database::{
    args,
    args_unwrap,
    database_env::{LoginDatabase, LoginPreparedStmts},
    DbDriver,
};
use azothacore_server::shared::{
    bnetrpc_zcompress,
    networking::{
        socket::{AddressOrName, Socket},
        socket_mgr::{ConnectionComponent, NewTcpConnection, RunStartTcpSocketTask, SocketReceiver},
    },
    realms::{
        realm_list::{JoinRealmError, RealmList},
        BnetRealmHandle,
    },
};
use bevy::{
    ecs::world::CommandQueue,
    prelude::{App, Commands, Component, Entity, FixedUpdate, IntoSystemConfigs, Query, Res, ResMut, SystemSet, Update, World},
};
use bnet_rpc::{
    bgs::protocol::{
        account::v1::{
            AccountFieldTags,
            AccountService,
            AccountState,
            GameAccountFieldTags,
            GameAccountState,
            GameLevelInfo,
            GameStatus,
            GetAccountStateRequest,
            GetAccountStateResponse,
            GetGameAccountStateRequest,
            GetGameAccountStateResponse,
            PrivacyInfo,
        },
        authentication::v1::{AuthenticationListener, AuthenticationService, LogonRequest, LogonResult, VerifyWebCredentialsRequest},
        challenge::v1::{ChallengeExternalRequest, ChallengeListener},
        channel::v1::ChannelService,
        connection::v1::{ConnectRequest, ConnectResponse, ConnectionService, DisconnectNotification, DisconnectRequest},
        friends::v1::FriendsService,
        game_utilities::v1::{ClientRequest, ClientResponse, GameUtilitiesService, GetAllValuesForAttributeRequest, GetAllValuesForAttributeResponse},
        presence::v1::PresenceService,
        report::v1::ReportService,
        resources::v1::ResourcesService,
        user_manager::v1::UserManagerService,
        Attribute,
        EntityId,
        Header,
        NoData,
        NoResponse,
        ProcessId,
        Variant,
    },
    BattlenetRpcErrorCode,
    BnetRpcResult,
    BnetRpcService,
    BnetServiceWrapper,
};
use bytes::{Buf, Bytes};
use prost::Message;
use rand::{rngs::OsRng, RngCore};
use sqlx::{Pool, Row};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    runtime::Handle,
    sync::OwnedSemaphorePermit,
};
use tracing::{debug, error, warn};

use crate::{config::AuthserverConfig, ssl_context::SslContext};

fn handle_game_utils_service_compress_response<T>(prefix: &[u8], data: &T) -> BnetRpcResult<Vec<u8>>
where
    T: ?Sized + serde::Serialize,
{
    let mut json = prefix.to_vec();
    if let Err(e) = serde_json::to_writer(&mut json, &data) {
        error!(target:"session", "unable to serialise {} to json, err={e}", String::from_utf8_lossy(prefix));
        return Err(BattlenetRpcErrorCode::UtilServerFailedToSerializeResponse);
    }

    match bnetrpc_zcompress(json) {
        Err(e) => {
            error!(target:"session", cause=?e, "unable to compress {}", String::from_utf8_lossy(prefix));
            Err(BattlenetRpcErrorCode::UtilServerFailedToSerializeResponse)
        },
        Ok(c) => Ok(c),
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AccountInfo {
    // void LoadResult(PreparedQueryResult result),
    id:                    u32,
    login:                 String,
    is_locked_to_ip:       bool,
    lock_country:          String,
    last_ip:               String,
    login_ticket_expiry:   u64,
    is_banned:             Option<bool>,
    is_permanently_banned: Option<bool>,
    #[sqlx(skip)]
    game_accounts:         BTreeMap<u32, GameAccountInfo>,
}

impl AccountInfo {
    fn is_banned(&self) -> bool {
        self.is_banned.is_some_and(|b| b)
    }

    fn is_permanently_banned(&self) -> bool {
        self.is_permanently_banned.is_some_and(|b| b)
    }

    async fn load_result(login_db: &Pool<DbDriver>, web_credentials: &[u8]) -> sqlx::Result<Option<Self>> {
        let Some(mut result) = LoginDatabase::sel_bnet_account_info_by_bnet_login_ticket::<_, Self>(login_db, args_unwrap!(web_credentials)).await? else {
            return Ok(None);
        };
        let game_account_infos =
            LoginDatabase::sel_game_account_info_by_bnet_login_ticket::<_, DbGameAccountInfo>(login_db, args_unwrap!(web_credentials)).await?;
        for db_gai in game_account_infos {
            result.game_accounts.entry(db_gai.id).or_insert_with(|| GameAccountInfo::load_result(db_gai));
        }
        Ok(Some(result))
    }
}

#[derive(Debug, Clone)]
struct LastPlayedCharacterInfo {
    realm_id:         BnetRealmHandle,
    character_name:   String,
    character_guid:   u64,
    last_played_time: u32,
}

#[derive(sqlx::FromRow)]
struct DbGameAccountInfo {
    id:                    u32,
    name:                  String,
    unban_date:            Option<u64>,
    is_permanently_banned: Option<bool>,
    security_level:        Option<u8>,
}

#[derive(Clone, Debug)]
struct GameAccountInfo {
    // void LoadResult(Field* fields);
    id:                     u32,
    name:                   String,
    display_name:           String,
    unban_date:             Option<u64>,
    is_banned:              bool,
    is_permanently_banned:  bool,
    _security_level:        AccountTypes,
    character_counts:       BTreeMap<u32, /*realmAddress*/ u8>,
    last_played_characters: BTreeMap<String /*: subRegion*/, LastPlayedCharacterInfo>,
}

impl GameAccountInfo {
    fn load_result(db_gai: DbGameAccountInfo) -> Self {
        let DbGameAccountInfo {
            id,
            name,
            unban_date,
            is_permanently_banned,
            security_level,
        } = db_gai;

        let is_permanently_banned = is_permanently_banned.is_some_and(|b| b);
        let is_banned = is_permanently_banned || unban_date.is_some_and(|ub| ub > unix_now().as_secs());
        let _security_level = security_level.map_or(AccountTypes::SecPlayer, |l| l.try_into().unwrap_or(AccountTypes::SecPlayer));

        let display_name = if let Some(sub_str_pos) = name.find('#') {
            format!("WoW{}", &name[..sub_str_pos + 1])
        } else {
            name.clone()
        };
        Self {
            id,
            name,
            display_name,
            unban_date,
            is_permanently_banned,
            is_banned,
            _security_level,
            character_counts: BTreeMap::new(),
            last_played_characters: BTreeMap::new(),
        }
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddBnetSessionSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BnetSessionReadPacketsSet;

pub fn bnet_session_handling_plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        handle_bnet_authserver_socket.run_if(az_startup_succeeded()).in_set(AddBnetSessionSet),
    )
    .add_systems(Update, read_handler.run_if(az_startup_succeeded()).in_set(BnetSessionReadPacketsSet));
}

fn handle_bnet_authserver_socket(
    mut commands: Commands,
    login_db: Res<LoginDatabase>,
    rt: Res<TokioRuntime>,
    ssl_ctx: Res<SslContext>,
    mut sock_recv: ResMut<SocketReceiver<SessionInner>>,
) {
    let ssl_ctx = ssl_ctx.clone();
    let login_db = login_db.clone();
    while let Ok(NewTcpConnection { permit, name, conn }) = sock_recv.0.try_recv() {
        let entity = commands.spawn_empty().id();
        let ssl_ctx = ssl_ctx.clone();
        let login_db = login_db.clone();

        let task = rt.spawn(async move {
            let sess = match SessionInner::start_from_tcp(login_db, ssl_ctx, permit, name, conn).await {
                Err(e) => {
                    error!(cause=?e, "error starting session from new TCP connections");
                    return CommandQueue::default();
                },
                Ok(s) => s,
            };

            let mut command_queue = CommandQueue::default();
            command_queue.push(move |world: &mut World| {
                world.entity_mut(entity).insert(sess).remove::<RunStartTcpSocketTask<SessionInner>>();
            });
            command_queue
        });
        commands.entity(entity).insert(RunStartTcpSocketTask::<SessionInner>::new(task));
    }
}

fn read_handler(
    mut commands: Commands,
    rt: Res<TokioRuntime>,
    cfg: Res<ConfigMgr<AuthserverConfig>>,
    login_db: Res<LoginDatabase>,
    realm_list: Res<RealmList>,
    mut sessions: Query<(Entity, &mut SessionInner)>,
) {
    for (e, mut inner) in &mut sessions {
        let packets = match inner.receive(None) {
            Err(err) => {
                error!(cause=?err, "error receiving packets");
                inner.close();
                commands.entity(e).remove::<SessionInner>();
                return;
            },
            Ok(p) => p,
        };
        let mut sess = Session {
            cfg:        &cfg,
            login_db:   &login_db,
            realm_list: &realm_list,
            inner:      &mut inner,
        };

        for AuthserverPacket { header, packet_buffer } in packets {
            let res: io::Result<()> = rt.block_on(async {
                if header.service_id != 0xFE {
                    sess.dispatch(header.service_hash(), header.token, header.method_id(), packet_buffer).await?;
                } else {
                    sess.receive(header.service_hash(), header.token, header.method_id(), packet_buffer).await?;
                }
                Ok(())
            });
            if let Err(err) = res {
                error!(cause=?err, caller_info=%sess.caller_info(), "session dispatch packets/receive packets err, terminating socket");
                sess.close();
                commands.entity(e).remove::<SessionInner>();
            }
        }
    }
}

pub struct AuthserverPacket {
    header:        Header,
    packet_buffer: Bytes,
}

impl DecodeValueFromBytes for AuthserverPacket {
    fn decode_from_bytes(buffer: &mut MessageBuffer) -> BufferResult<Self>
    where
        Self: std::marker::Sized,
    {
        // // ReadHeaderLengthHandler
        // let header_length = u16::from_be_bytes(self.inner.receive(2).await?.to_vec().try_into().unwrap());
        // // ReadHeaderHandler
        // let header = Header::decode(self.inner.receive(header_length.into()).await?)?;
        // // ReadDataHandler
        // let packet_buffer = self.inner.receive(header.size().try_into().unwrap()).await?;
        let mut previous_section_size = 0;
        let mut expected_packet_size = 2;
        let current_len = buffer.len();
        if current_len < expected_packet_size {
            return Err(BufferDecodeError::InsufficientBytes {
                have:   current_len,
                wanted: expected_packet_size,
            });
        }
        // ReadHeaderLengthHandler
        let header_length = u16::from_be_bytes(buffer[previous_section_size..expected_packet_size].try_into().unwrap());
        previous_section_size = expected_packet_size;
        expected_packet_size += <_ as Into<usize>>::into(header_length);
        if current_len < expected_packet_size {
            return Err(BufferDecodeError::InsufficientBytes {
                have:   current_len,
                wanted: expected_packet_size,
            });
        }
        // ReadHeaderHandler
        let header = match Header::decode(&buffer[previous_section_size..expected_packet_size]) {
            Err(e) => {
                return Err(BufferDecodeError::UnexpectedDecode(e.to_string()));
            },
            Ok(h) => h,
        };
        // ReadDataHandler
        previous_section_size = expected_packet_size;
        let packet_buffer_size = header.size().try_into().unwrap();
        expected_packet_size += packet_buffer_size;
        if current_len < expected_packet_size {
            return Err(BufferDecodeError::InsufficientBytes {
                have:   current_len,
                wanted: expected_packet_size,
            });
        }
        // Shift buffer forward.
        buffer.advance(previous_section_size);
        let packet_buffer = buffer.split_to(packet_buffer_size).freeze();
        Ok(AuthserverPacket { header, packet_buffer })
    }
}

pub struct Session<'a> {
    login_db:   &'a LoginDatabase,
    cfg:        &'a AuthserverConfig,
    realm_list: &'a RealmList,
    inner:      &'a mut SessionInner,
}

impl Deref for Session<'_> {
    type Target = SessionInner;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
impl DerefMut for Session<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

#[derive(Component)]
pub struct SessionInner {
    // Rest of Session info.
    /// Contains a mapping of tokens expecting a return from a previous client
    /// call to service hash and method_id
    ///
    response_callbacks: Mutex<BTreeMap<u32, (u32, u32)>>,
    _permit:            Option<OwnedSemaphorePermit>,
    socket:             Socket<AuthserverPacket>,
    /// the current session's account information. If this is set, then we can treat it as _authed=True
    /// like in TC/AC
    account_info:       OnceLock<AccountInfo>,
    /// Game account info, saved during Command_RealmListTicketRequest_v1_b9
    /// for now its a cloned value from `account_info`
    game_account_info:  OnceLock<GameAccountInfo>,
    locale:             OnceLock<Locale>,
    os:                 OnceLock<String>,
    build:              AtomicU32,
    ip_country:         OnceLock<String>,
    client_secret:      OnceLock<[u8; 32]>,
    request_token:      AtomicU32,
}

deref_boilerplate!(SessionInner, Socket<AuthserverPacket>, socket);

impl ConnectionComponent for SessionInner {}

fn map_err_to_denied<E: std::error::Error>(e: E) -> BattlenetRpcErrorCode {
    error!(target:"session::rpc", cause=?e, "error when making sql queries in bnet session. mapped to denied");
    BattlenetRpcErrorCode::Denied
}

fn map_err_to_internal<E: std::error::Error>(e: E) -> BattlenetRpcErrorCode {
    error!(target:"session::rpc", cause=?e, "error when making sql queries in bnet session. mapped to Internal");
    BattlenetRpcErrorCode::Internal
}

impl Session<'_> {
    async fn receive(&mut self, service_hash: u32, token: u32, method_id: u32, _packet_buffer: Bytes) -> io::Result<()> {
        let (stored_service_hash, stored_method_id) = match self.response_callbacks.lock().unwrap().remove(&token) {
            None => return Ok(()),
            Some(d) => d,
        };
        if stored_service_hash != service_hash {
            warn!(target:"session::rpc", "stored service_hash do not match with the ones received: stored=0x{stored_service_hash:X}, got=0x{service_hash:X}");
        }
        if stored_method_id != method_id {
            warn!(target:"session::rpc", "stored method_id do not match with the ones received: stored={stored_method_id}, got={method_id}");
        }

        // // TODO: Add the respective service's `receive_client_response` in order to handle callbacks`
        // self.receive_client_response(method_id, message)?;
        Ok(())
    }

    async fn dispatch(&self, service_hash: u32, token: u32, method_id: u32, packet_buffer: Bytes) -> io::Result<()> {
        match service_hash {
            c if c == AccountService::service_hash(self) => AccountService::call_server_method(self, token, method_id, packet_buffer).await?,
            c if c == AuthenticationService::service_hash(self) => AuthenticationService::call_server_method(self, token, method_id, packet_buffer).await?,
            c if c == ChannelService::service_hash(self) => ChannelService::call_server_method(self, token, method_id, packet_buffer).await?,
            c if c == ConnectionService::service_hash(self) => ConnectionService::call_server_method(self, token, method_id, packet_buffer).await?,
            c if c == FriendsService::service_hash(self) => FriendsService::call_server_method(self, token, method_id, packet_buffer).await?,
            c if c == GameUtilitiesService::service_hash(self) => GameUtilitiesService::call_server_method(self, token, method_id, packet_buffer).await?,
            c if c == PresenceService::service_hash(self) => PresenceService::call_server_method(self, token, method_id, packet_buffer).await?,
            c if c == ReportService::service_hash(self) => ReportService::call_server_method(self, token, method_id, packet_buffer).await?,
            c if c == ResourcesService::service_hash(self) => ResourcesService::call_server_method(self, token, method_id, packet_buffer).await?,
            c if c == UserManagerService::service_hash(self) => UserManagerService::call_server_method(self, token, method_id, packet_buffer).await?,
            c => {
                warn!(target:"session::rpc", caller_info=self.caller_info(), "client tried to call invalid service {:?}", c);
            },
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, target="session", fields(caller_info=self.caller_info()))]
    async fn verify_web_credentials(&self, web_credentials: &[u8]) -> BnetRpcResult<NoData> {
        if web_credentials.is_empty() {
            return Err(BattlenetRpcErrorCode::Denied);
        }
        let Some(mut account_info) = AccountInfo::load_result(self.login_db, web_credentials).await.map_err(map_err_to_denied)? else {
            return Err(BattlenetRpcErrorCode::Denied);
        };

        if account_info.login_ticket_expiry < unix_now().as_secs() {
            return Err(BattlenetRpcErrorCode::TimedOut);
        }
        let character_counts_result = LoginDatabase::sel_bnet_character_counts_by_bnet_id(&**self.login_db, args_unwrap!(account_info.id))
            .await
            .map_err(map_err_to_internal)?;
        for fields in character_counts_result {
            let account_id = fields.get(0);
            let game_account = match account_info.game_accounts.get_mut(&account_id) {
                // should already be inside the account info.
                None => continue,
                Some(a) => a,
            };
            game_account
                .character_counts
                .entry(BnetRealmHandle::new(fields.get(3), fields.get(4), fields.get(2)).get_address())
                .or_insert(fields.get(1));
        }
        let last_player_characters_result = LoginDatabase::sel_bnet_last_player_characters(&**self.login_db, args_unwrap!(account_info.id))
            .await
            .map_err(map_err_to_internal)?;
        for fields in last_player_characters_result {
            let realm_id = BnetRealmHandle::new(fields.get(1), fields.get(2), fields.get(3));
            let game_account = match account_info.game_accounts.get_mut(&fields.get(0)) {
                None => continue,
                Some(a) => a,
            };
            game_account
                .last_played_characters
                .entry(realm_id.get_sub_region_address())
                .or_insert(LastPlayedCharacterInfo {
                    realm_id,
                    character_name: fields.get(4),
                    character_guid: fields.get(5),
                    last_played_time: fields.get(6),
                });
        }
        let ip_address = self.remote_name().ip_str_or_name();

        // If the IP is 'locked', check that the player comes indeed from the correct IP address
        if account_info.is_locked_to_ip {
            debug!(
                target:"session",
                account = account_info.login,
                last_ip = account_info.last_ip,
                ip_address = ip_address,
                "[Session::HandleVerifyWebCredentials] Account is locked to IP"
            );
            if account_info.last_ip != ip_address {
                return Err(BattlenetRpcErrorCode::RiskAccountLocked);
            }
        } else {
            debug!(
                target:"session",
                account = account_info.login,
                "[Session::HandleVerifyWebCredentials] Account is not locked to ip"
            );
            match account_info.lock_country.as_str() {
                "" | "00" => {
                    debug!(target:"session",
                        account = account_info.login,
                        "[Session::HandleVerifyWebCredentials] Account is not locked to country"
                    );
                },
                accnt_lock_cty => {
                    if let Some(ip_cty) = self.ip_country.get() {
                        debug!(target:"session",
                            account = account_info.login,
                            account_country = accnt_lock_cty,
                            ip_country = ip_cty,
                            "[Session::HandleVerifyWebCredentials] Account is locked to country"
                        );
                        if !ip_cty.eq(&accnt_lock_cty) {
                            return Err(BattlenetRpcErrorCode::RiskAccountLocked);
                        }
                    }
                },
            };
        }
        // If the account is banned, reject the logon attempt
        if account_info.is_banned() {
            if account_info.is_permanently_banned() {
                debug!(target:"session",
                    account = account_info.login,
                    "[Session::HandleVerifyWebCredentials] Banned account tried to login!",
                );
                return Err(BattlenetRpcErrorCode::GameAccountBanned);
            } else {
                debug!(target:"session",
                    account = account_info.login,
                    "[Session::HandleVerifyWebCredentials] Temporarily banned account tried to login!",
                );
                return Err(BattlenetRpcErrorCode::GameAccountSuspended);
            }
        }
        let game_account_id = account_info
            .game_accounts
            .values()
            .map(|ga| EntityId {
                low:  ga.id.into(),
                high: 0x200000200576F57,
            })
            .collect();

        let mut session_key = vec![0; 64];
        OsRng.fill_bytes(&mut session_key);
        let logon_result = LogonResult {
            error_code: 0,
            account_id: Some(EntityId {
                low:  account_info.id.into(),
                high: 0x100000000000000,
            }),
            geoip_country: self.ip_country.get().cloned(),
            game_account_id,
            session_key: Some(session_key.into()),
            ..Default::default()
        };

        if let Err(account_info) = self.account_info.set(account_info) {
            error!(target:"session", attempted=?account_info, current=?self.account_info.get(), "error setting new account info as one has already been set before");
        }

        self.on_logon_complete(logon_result).await.map_err(map_err_to_internal)?;

        Ok(NoData {})
    }
}

impl SessionInner {
    async fn start_from_tcp(
        login_db: LoginDatabase,
        ssl_ctx: SslContext,
        permit: Option<OwnedSemaphorePermit>,
        addr: AddressOrName,
        tcp_conn: TcpStream,
    ) -> AzResult<Self>
    where
        Self: std::marker::Sized,
    {
        let conn = ssl_ctx.accept(tcp_conn).await?;
        let (rd, wr) = tokio::io::split(conn);
        Self::start(login_db, permit, addr, rd, wr).await
    }

    async fn start<R, W>(login_db: LoginDatabase, _permit: Option<OwnedSemaphorePermit>, name: AddressOrName, rd: R, wr: W) -> AzResult<Self>
    where
        R: AsyncRead + Unpin + Send + Sync + 'static,
        W: AsyncWrite + Unpin + Send + Sync + 'static,
    {
        // CheckIpCallback routine
        let ip_address = name.ip_str_or_name();
        // Verify that this IP is not in the ip_banned table
        LoginDatabase::del_expired_ip_bans(&*login_db, args!()?).await?;
        for fields in LoginDatabase::sel_ip_info(&*login_db, args!(ip_address)?).await? {
            let banned = fields.get::<u32, _>("banned") != 0;
            if banned {
                let e = az_error!("[CheckIpCallback] Banned ip '{}' tries to login!", name);
                debug!(target:"session", cause=?e);
                return Err(e);
            }
        }
        // begin AsyncRead => ReadHandler routine
        let socket = Socket::<AuthserverPacket>::new(&Handle::current(), name, rd, wr);

        let s = SessionInner {
            socket,
            _permit,
            response_callbacks: Mutex::new(BTreeMap::new()),
            // socket.remote_name(): name,
            account_info: OnceLock::new(),
            game_account_info: OnceLock::new(),
            build: AtomicU32::new(0),
            locale: OnceLock::new(),
            os: OnceLock::new(),
            client_secret: OnceLock::new(),
            ip_country: OnceLock::new(),
            request_token: AtomicU32::new(0),
        };
        debug!(target:"session", caller=%s.remote_name(), "Accepted connection");
        Ok(s)
    }
}

impl BnetRpcService for Session<'_> {
    fn caller_info(&self) -> String {
        let mut stream = format!("[{}", self.remote_name());
        {
            if let Some(ai) = self.account_info.get() {
                stream.push_str(&format!(", Account: {}", ai.login));
            }
        }
        {
            if let Some(gai) = self.game_account_info.get() {
                stream.push_str(&format!(", Game account: {}", gai.name));
            }
        }
        stream.push(']');
        stream
    }

    async fn send_server_response<M>(&self, response: BnetServiceWrapper<M>) -> std::io::Result<()>
    where
        M: prost::Message,
    {
        let BnetServiceWrapper { token, result, .. } = response;

        let (status, response) = match result {
            Err(e) => (Some(e.into()), None),
            Ok(p) => (None, Some(p)),
        };

        let header = Header {
            token,
            status,
            service_id: 0xFE,
            size: response.as_ref().map(|p| {
                p.encoded_len()
                    .try_into()
                    .unwrap_or_else(|e| panic!("send size error convert. should never fail here; err={e}"))
            }),
            ..Default::default()
        };
        let header_size = u16::try_from(header.encoded_len())
            .unwrap_or_else(|e| panic!("send header size error size convert. should never fail here; err={e}"))
            .to_be_bytes();
        let mut packet = Vec::with_capacity(header_size.len() + header.encoded_len() + response.as_ref().map_or(0, |r| r.encoded_len()));
        packet.write_all(&header_size)?;
        header.encode(&mut packet)?;
        if let Some(r) = response {
            r.encode(&mut packet)?;
        }
        // AsyncWrite
        self.write(packet)?;

        Ok(())
    }

    async fn make_client_request<M>(&self, request: BnetServiceWrapper<M>) -> std::io::Result<()>
    where
        M: prost::Message,
    {
        let BnetServiceWrapper {
            service_hash,
            method_id,
            token,
            result,
        } = request;
        let request = result.expect("client request should always be set, check impl of `pre_send_store_client_request`");

        let header = Header {
            service_id: 0,
            service_hash: Some(service_hash),
            method_id: Some(method_id),
            size: request.encoded_len().try_into().ok(),
            token,
            ..Default::default()
        };
        let header_size = u16::try_from(header.encoded_len())
            .unwrap_or_else(|e| panic!("send header size error size convert. should never fail here; err={e}"))
            .to_be_bytes();
        let mut packet = Vec::with_capacity(header_size.len() + header.encoded_len() + request.encoded_len());
        packet.write_all(&header_size)?;
        header.encode(&mut packet)?;
        request.encode(&mut packet)?;
        // AsyncWrite
        self.write(packet)?;

        Ok(())
    }

    async fn pre_send_store_client_request<M>(&self, service_hash: u32, method_id: u32, request: M) -> io::Result<BnetServiceWrapper<M>>
    where
        M: prost::Message,
    {
        let token = self.request_token.fetch_add(1, Ordering::SeqCst);
        self.response_callbacks.lock().unwrap().entry(token).or_insert((service_hash, method_id));
        Ok(BnetServiceWrapper {
            service_hash,
            method_id,
            token,
            result: Ok(request),
        })
    }
}

impl AuthenticationListener for Session<'_> {
    const USE_ORIGINAL_HASH: bool = true;
}

impl ChallengeListener for Session<'_> {
    const USE_ORIGINAL_HASH: bool = true;
}

impl AccountService for Session<'_> {
    const USE_ORIGINAL_HASH: bool = true;

    async fn handle_srv_req_get_account_state(&self, request: GetAccountStateRequest) -> BnetRpcResult<GetAccountStateResponse>
    where
        Self: Sync,
    {
        if self.account_info.get().is_none() {
            return Err(BattlenetRpcErrorCode::Denied);
        }
        let mut response = GetAccountStateResponse::default();
        if matches!(request.options, Some(opts) if opts.field_privacy_info()) {
            response.state = Some(AccountState {
                privacy_info: Some(PrivacyInfo {
                    is_using_rid: Some(false),
                    is_visible_for_view_friends: Some(false),
                    is_hidden_from_friend_finder: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            });
            response.tags = Some(AccountFieldTags {
                privacy_info_tag: Some(0xD7CA834D),
                ..Default::default()
            });
        }
        Ok(response)
    }

    async fn handle_srv_req_get_game_account_state(&self, request: GetGameAccountStateRequest) -> BnetRpcResult<GetGameAccountStateResponse>
    where
        Self: Sync,
    {
        let Some(accnt_info) = self.account_info.get() else {
            return Err(BattlenetRpcErrorCode::Denied);
        };
        let Some(gai) = &request.game_account_id else {
            return Ok(GetGameAccountStateResponse::default());
        };

        let mut game_account_state = GameAccountState::default();
        let mut tags = GameAccountFieldTags::default();

        let game_accounts = &accnt_info.game_accounts;
        if matches!(&request.options, Some(opts) if opts.field_game_level_info()) {
            if let Some(ga) = game_accounts.get(&gai.low.try_into().unwrap()) {
                game_account_state.game_level_info = Some(GameLevelInfo {
                    name: Some(ga.display_name.clone()),
                    program: Some(5730135), // WoW
                    ..Default::default()
                });
            }
            tags.game_level_info_tag = Some(0x5C46D483);
        }

        if matches!(&request.options, Some(opts) if opts.field_game_status()) {
            let mut gs = GameStatus {
                program: Some(5730135), // WoW
                ..Default::default()
            };
            if let Some(ga) = game_accounts.get(&gai.low.try_into().unwrap()) {
                gs.is_suspended = Some(ga.is_banned);
                gs.is_banned = Some(ga.is_permanently_banned);
                gs.suspension_expires = ga.unban_date.map(|ud| ud * 1000000);
            }
            game_account_state.game_status = Some(gs);
            tags.game_status_tag = Some(0x98B75F99);
        }
        Ok(GetGameAccountStateResponse {
            tags:  Some(tags),
            state: Some(game_account_state),
        })
    }
}

impl AuthenticationService for Session<'_> {
    const USE_ORIGINAL_HASH: bool = true;

    #[tracing::instrument(
        skip(self),
        target = "session",
        fields(caller_info = self.caller_info())
    )]
    async fn handle_srv_req_logon(&self, request: LogonRequest) -> BnetRpcResult<NoData>
    where
        Self: Sync,
    {
        if request.program() != "WoW" {
            debug!(
                target:"session",
                "[Battlenet::LogonRequest] attempted to log in with game other than WoW (using {})!",
                request.program()
            );
            return Err(BattlenetRpcErrorCode::BadProgram);
        }

        if request.platform() != "Win" && request.platform() != "Wn64" && request.platform() != "Mc64" {
            debug!(
                target:"session",
                "[Battlenet::LogonRequest] attempted to log in from an unsupported platform (using {})!",
                request.platform()
            );
            return Err(BattlenetRpcErrorCode::BadPlatform);
        }

        let locale = match request.locale().parse::<Locale>() {
            Err(e) => {
                debug!(target:"session",cause=?e, "[Battlenet::LogonRequest] attempted to log in with unsupported locale (using {})!", request.locale());
                return Err(BattlenetRpcErrorCode::BadLocale);
            },
            Ok(l) => l,
        };

        if let Err(locale) = self.locale.set(locale) {
            error!(target:"session", attempted=?locale, current=?self.locale.get(), "error setting new locale as one has already been set before");
        }
        if let Err(os) = self.os.set(request.platform().to_string()) {
            error!(target:"session", attempted=?os, current=?self.os.get(), "error setting new os as one has already been set before");
        }
        self.build.store(request.application_version().try_into().unwrap(), Ordering::SeqCst);

        if let Some(web_creds) = request.cached_web_credentials {
            return self.verify_web_credentials(web_creds.as_ref()).await;
        }

        let endpoint = self.cfg.login_rest_get_address_for_client(self.remote_name());
        let external_challenge = ChallengeExternalRequest {
            payload_type: Some("web_auth_url".to_string()),
            payload: Some(format!("https://{endpoint}/bnetserver/login/").into()),
            ..Default::default()
        };
        if let Err(e) = self.on_external_challenge(external_challenge).await {
            debug!(target:"session",cause=?e, "[Battlenet::LogonRequest] error sending external challenge");
        }

        Ok(NoData {})
    }

    async fn handle_srv_req_verify_web_credentials(&self, request: VerifyWebCredentialsRequest) -> BnetRpcResult<NoData>
    where
        Self: Sync,
    {
        self.verify_web_credentials(request.web_credentials()).await
    }
}

// impl ChallengeService for Session {
//     const USE_ORIGINAL_HASH: bool = true;
// }

impl ChannelService for Session<'_> {
    const USE_ORIGINAL_HASH: bool = true;
}

impl ConnectionService for Session<'_> {
    const USE_ORIGINAL_HASH: bool = true;

    async fn handle_srv_req_connect(&self, request: ConnectRequest) -> BnetRpcResult<ConnectResponse>
    where
        Self: Sync,
    {
        let now = unix_now();
        Ok(ConnectResponse {
            use_bindless_rpc: Some(request.use_bindless_rpc()),
            client_id: request.client_id,
            server_id: ProcessId {
                label: process::id(),
                epoch: now.as_secs().try_into().unwrap(),
            },
            server_time: Some(now.as_millis().try_into().unwrap()),
            ..Default::default()
        })
    }

    async fn handle_srv_req_keep_alive(&self, _request: NoData) -> BnetRpcResult<NoResponse>
    where
        Self: Sync,
    {
        Ok(NoResponse {})
    }

    async fn handle_srv_req_request_disconnect(&self, request: DisconnectRequest) -> BnetRpcResult<NoResponse>
    where
        Self: Sync,
    {
        let disconnect_notification = DisconnectNotification {
            error_code: request.error_code,
            ..Default::default()
        };
        _ = self.force_disconnect(disconnect_notification).await;
        self.inner.close();
        debug!(target:"session", "closed session due to request to be disconnected");
        Ok(NoResponse {})
    }
}

impl FriendsService for Session<'_> {
    const USE_ORIGINAL_HASH: bool = true;
}

impl GameUtilitiesService for Session<'_> {
    const USE_ORIGINAL_HASH: bool = true;

    async fn handle_srv_req_process_client_request(&self, request: ClientRequest) -> BnetRpcResult<ClientResponse>
    where
        Self: Sync,
    {
        if self.account_info.get().is_none() {
            return Err(BattlenetRpcErrorCode::Denied);
        };

        let mut command = None;
        let mut params = BTreeMap::new();
        for attr in request.attribute {
            if attr.name.starts_with("Command_") {
                command = Some(attr.name.clone())
            }
            params.entry(attr.name).or_insert_with(|| attr.value);
        }

        let Some(command) = command else {
            error!(target:"session::rpc", client=self.caller_info(), "client sent ClientRequest with no command.");
            return Err(BattlenetRpcErrorCode::RpcMalformedRequest);
        };
        let resp = match command.as_str() {
            "Command_RealmListTicketRequest_v1_b9" => self.get_realm_list_ticket(params).await,
            "Command_LastCharPlayedRequest_v1_b9" => self.get_last_char_played(params).await,
            "Command_RealmListRequest_v1_b9" => self.get_realm_list(params).await,
            "Command_RealmJoinRequest_v1_b9" => self.join_realm(params).await,
            cmd => {
                error!(
                    target:"session::rpc",
                    client = self.caller_info(),
                    "client sent ClientRequest with unknown command {cmd}"
                );
                return Err(BattlenetRpcErrorCode::RpcNotImplemented);
            },
        };
        resp
    }

    async fn handle_srv_req_get_all_values_for_attribute(&self, request: GetAllValuesForAttributeRequest) -> BnetRpcResult<GetAllValuesForAttributeResponse>
    where
        Self: Sync,
    {
        if self.account_info.get().is_none() {
            return Err(BattlenetRpcErrorCode::Denied);
        };

        if request.attribute_key() == "Command_RealmListRequest_v1_b9" {
            let response = GetAllValuesForAttributeResponse {
                attribute_value: self
                    .realm_list
                    .sub_regions
                    .iter()
                    .map(|sub_region| Variant {
                        string_value: Some(sub_region.clone()),
                        ..Default::default()
                    })
                    .collect(),
            };
            return Ok(response);
        }
        Err(BattlenetRpcErrorCode::RpcNotImplemented)
    }
}

impl PresenceService for Session<'_> {
    const USE_ORIGINAL_HASH: bool = true;
}

impl ReportService for Session<'_> {
    const USE_ORIGINAL_HASH: bool = true;
}

impl ResourcesService for Session<'_> {
    const USE_ORIGINAL_HASH: bool = true;
}

impl UserManagerService for Session<'_> {
    const USE_ORIGINAL_HASH: bool = true;
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RealmListTicketIdentity {
    #[serde(rename = "gameAccountID")]
    game_account_id:     u32,
    #[serde(rename = "gameAccountRegion")]
    game_account_region: u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct ClientVersion {
    #[serde(rename = "versionMajor")]
    version_major:    u32,
    #[serde(rename = "versionMinor")]
    version_minor:    u32,
    #[serde(rename = "versionRevision")]
    version_revision: u32,
    #[serde(rename = "versionBuild")]
    version_build:    u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct ClientInformation {
    #[serde(rename = "platform")]
    platform:           u32,
    #[serde(rename = "buildVariant")]
    build_variant:      String,
    #[serde(rename = "type")]
    typ:                u32,
    #[serde(rename = "timeZone")]
    time_zone:          String,
    #[serde(rename = "currentTime")]
    current_time:       u32,
    #[serde(rename = "textLocale")]
    text_locale:        u32,
    #[serde(rename = "audioLocale")]
    audio_locale:       u32,
    #[serde(rename = "versionDataBuild")]
    version_data_build: u32,
    version:            ClientVersion,
    secret:             [u8; 32],
    #[serde(rename = "clientArch")]
    client_arch:        u32,
    #[serde(rename = "systemVersion")]
    system_version:     String,
    #[serde(rename = "platformType")]
    platform_type:      u32,
    #[serde(rename = "systemArch")]
    system_arch:        u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct RealmListTicketClientInformation {
    info: ClientInformation,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct RealmCharacterCountEntry {
    #[serde(rename = "wowRealmAddress")]
    wow_realm_address: u32,
    count:             u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct RealmCharacterCountList {
    counts: Vec<RealmCharacterCountEntry>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct IPAddress {
    ip:   String,
    port: u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct RealmIPAddressFamily {
    family:    u32,
    addresses: Vec<IPAddress>,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct RealmListServerIPAddresses {
    families: Vec<RealmIPAddressFamily>,
}

fn json_from_blob(blob: &[u8]) -> Option<&[u8]> {
    let (colon_idx, _) = blob.iter().enumerate().find(|(_, c)| **c == b':')?;
    // Take from after the colon, and a char the end, as last char is always a NUL char i.e. "\0"
    Some(&blob[colon_idx + 1..blob.len() - 1])
}

impl Session<'_> {
    async fn get_realm_list_ticket(&self, params: BTreeMap<String, Variant>) -> BnetRpcResult<ClientResponse> {
        let Some(accnt_info) = self.account_info.get() else {
            return Err(BattlenetRpcErrorCode::WowServicesDeniedRealmListTicket);
        };
        if let Some(identity) = params.get("Param_Identity") {
            let blob = identity.blob_value();
            let Some(json_start) = json_from_blob(blob) else {
                return Err(BattlenetRpcErrorCode::InvalidArgs);
            };
            match serde_json::from_slice(json_start) {
                Err(e) => {
                    warn!(target:"session", cause=?e, identity_val=String::from_utf8_lossy(json_start).to_string(), current=?accnt_info, "error serialising params identity from json");
                },
                Ok(RealmListTicketIdentity { game_account_id, .. }) => {
                    if let Some(gai) = accnt_info.game_accounts.get(&game_account_id) {
                        if let Err(new_val) = self.game_account_info.set(gai.clone()) {
                            warn!(target:"session", new_val=?new_val, current=?accnt_info, "error overwriting existing game account");
                            return Err(BattlenetRpcErrorCode::WowServicesDeniedRealmListTicket);
                        }
                    }
                },
            };
        }
        let Some(game_account_info) = self.game_account_info.get() else {
            return Err(BattlenetRpcErrorCode::UtilServerInvalidIdentityArgs);
        };
        if game_account_info.is_permanently_banned {
            return Err(BattlenetRpcErrorCode::GameAccountBanned);
        }
        if game_account_info.is_banned {
            return Err(BattlenetRpcErrorCode::GameAccountSuspended);
        }

        if let Some(client_info) = params.get("Param_ClientInfo") {
            let blob = client_info.blob_value();
            let Some(json_start) = json_from_blob(blob) else {
                return Err(BattlenetRpcErrorCode::InvalidArgs);
            };
            match serde_json::from_slice(json_start) {
                Err(e) => {
                    warn!(target:"session", cause=?e, val=String::from_utf8_lossy(json_start).to_string(), current=?accnt_info, "error serialising params identity from json");
                },
                Ok(RealmListTicketClientInformation {
                    info: ClientInformation { secret, .. },
                }) => {
                    if let Err(new_val) = self.client_secret.set(secret) {
                        warn!(target:"session", new_val=?new_val, current=?accnt_info, "error overwriting existing client secret");
                        return Err(BattlenetRpcErrorCode::WowServicesDeniedRealmListTicket);
                    }
                },
            };
        }
        if self.client_secret.get().is_none() {
            return Err(BattlenetRpcErrorCode::WowServicesDeniedRealmListTicket);
        }

        LoginDatabase::upd_bnet_last_login_info(
            &**self.login_db,
            args_unwrap!(
                self.remote_name().ip_str_or_name(),
                self.locale.get().map_or(Locale::enUS as u32, |l| *l as u32),
                self.os.get(),
                accnt_info.id
            ),
        )
        .await
        .map_err(map_err_to_internal)?;

        Ok(ClientResponse {
            attribute: vec![Attribute {
                name:  "Param_RealmListTicket".to_string(),
                value: Variant {
                    blob_value: Some(b"AuthRealmListTicket".to_vec().into()),
                    ..Default::default()
                },
            }],
        })
    }

    async fn get_last_char_played(&self, params: BTreeMap<String, Variant>) -> BnetRpcResult<ClientResponse> {
        let Some(sub_region) = params.get("Command_LastCharPlayedRequest_v1_b9") else {
            return Err(BattlenetRpcErrorCode::UtilServerUnknownRealm);
        };
        let mut response = ClientResponse { attribute: vec![] };
        let Some(gai) = self.game_account_info.get() else { return Ok(response) };

        if let Some(last_player_char) = gai.last_played_characters.get(sub_region.string_value()) {
            let Some(realm_entry) = self
                .realm_list
                .get_realm_entry_json(&last_player_char.realm_id, self.build.load(Ordering::SeqCst))
            else {
                return Err(BattlenetRpcErrorCode::UtilServerFailedToSerializeResponse);
            };
            let compressed = handle_game_utils_service_compress_response(b"JamJSONRealmEntry:", &realm_entry)?;
            response.attribute = vec![
                Attribute {
                    name:  "Param_RealmEntry".to_string(),
                    value: Variant {
                        blob_value: Some(compressed.into()),
                        ..Default::default()
                    },
                },
                Attribute {
                    name:  "Param_CharacterName".to_string(),
                    value: Variant {
                        string_value: Some(last_player_char.character_name.clone()),
                        ..Default::default()
                    },
                },
                Attribute {
                    name:  "Param_CharacterGUID".to_string(),
                    value: Variant {
                        blob_value: Some(last_player_char.character_guid.to_le_bytes().to_vec().into()),
                        ..Default::default()
                    },
                },
                Attribute {
                    name:  "Param_LastPlayedTime".to_string(),
                    value: Variant {
                        int_value: Some(last_player_char.last_played_time.into()),
                        ..Default::default()
                    },
                },
            ];
        }

        Ok(response)
    }

    async fn get_realm_list(&self, params: BTreeMap<String, Variant>) -> BnetRpcResult<ClientResponse> {
        let Some(game_account_info) = self.game_account_info.get() else {
            return Err(BattlenetRpcErrorCode::UserServerBadWowAccount);
        };
        let sub_region_id = if let Some(sub_region) = params.get("Command_RealmListRequest_v1_b9") {
            sub_region.string_value().to_string()
        } else {
            "".to_string()
        };

        let realm_list = self.realm_list.get_realm_list(self.build.load(Ordering::SeqCst), &sub_region_id);
        let realm_list_compressed = handle_game_utils_service_compress_response(b"JSONRealmListUpdates:", &realm_list)?;

        let realm_character_counts = RealmCharacterCountList {
            counts: game_account_info
                .character_counts
                .iter()
                .map(|character_count| RealmCharacterCountEntry {
                    wow_realm_address: *character_count.0,
                    count:             (*character_count.1).into(),
                })
                .collect(),
        };
        let realm_character_counts_compressed = handle_game_utils_service_compress_response(b"JSONRealmCharacterCountList:", &realm_character_counts)?;
        Ok(ClientResponse {
            attribute: vec![
                Attribute {
                    name:  "Param_RealmList".to_string(),
                    value: Variant {
                        blob_value: Some(realm_list_compressed.into()),
                        ..Default::default()
                    },
                },
                Attribute {
                    name:  "Param_CharacterCountList".to_string(),
                    value: Variant {
                        blob_value: Some(realm_character_counts_compressed.into()),
                        ..Default::default()
                    },
                },
            ],
        })
    }

    async fn join_realm(&self, params: BTreeMap<String, Variant>) -> BnetRpcResult<ClientResponse> {
        let Some(game_account_info) = self.game_account_info.get() else {
            return Err(BattlenetRpcErrorCode::UserServerBadWowAccount);
        };
        let Some(realm_address) = params.get("Param_RealmAddress") else {
            return Err(BattlenetRpcErrorCode::WowServicesInvalidJoinTicket);
        };
        let Some(client_secret) = self.client_secret.get() else {
            return Err(BattlenetRpcErrorCode::WowServicesDeniedRealmListTicket);
        };

        fn map_join_realm_err(e: JoinRealmError) -> BattlenetRpcErrorCode {
            match e {
                JoinRealmError::NotPermitted => BattlenetRpcErrorCode::UserServerNotPermittedOnRealm,
                JoinRealmError::UnknownRealm => BattlenetRpcErrorCode::UtilServerUnknownRealm,
                JoinRealmError::General => BattlenetRpcErrorCode::UtilServerFailedToSerializeResponse,
            }
        }
        let server_addresses = self
            .realm_list
            .retrieve_realm_list_server_ip_addresses(realm_address.uint_value() as u32, self.remote_name(), self.build.load(Ordering::SeqCst))
            .map_err(map_join_realm_err)?;
        let server_addresses = handle_game_utils_service_compress_response(b"JSONRealmListServerIPAddresses:", &server_addresses)?;
        let server_secret = RealmList::join_realm(
            (**self.login_db).clone(),
            self.remote_name(),
            client_secret,
            self.locale.get().cloned().unwrap_or(Locale::enUS),
            self.os.get().unwrap(),
            &game_account_info.name,
        )
        .await
        .map_err(map_join_realm_err)?;
        Ok(ClientResponse {
            attribute: vec![
                Attribute {
                    name:  "Param_RealmJoinTicket".to_string(),
                    value: Variant {
                        blob_value: Some(game_account_info.name.as_bytes().to_vec().into()),
                        ..Default::default()
                    },
                },
                Attribute {
                    name:  "Param_ServerAddresses".to_string(),
                    value: Variant {
                        blob_value: Some(server_addresses.into()),
                        ..Default::default()
                    },
                },
                Attribute {
                    name:  "Param_JoinSecret".to_string(),
                    value: Variant {
                        blob_value: Some(server_secret.to_vec().into()),
                        ..Default::default()
                    },
                },
            ],
        })
    }
}
