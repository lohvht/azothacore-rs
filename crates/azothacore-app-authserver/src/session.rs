use std::{
    collections::BTreeMap,
    io::{self, Write},
    process,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
        Mutex,
        OnceLock,
    },
};

use azothacore_common::{
    utils::{unix_now, SharedFromSelf, SharedFromSelfBase},
    AccountTypes,
    AzError,
    AzResult,
    Locale,
};
use azothacore_server::{
    database::{
        database_env::{LoginDatabase, LoginPreparedStmts},
        params,
    },
    shared::{
        bnetrpc_zcompress,
        networking::socket::{AddressOrName, Socket, SocketWrappper},
        realms::{
            realm_list::{JoinRealmError, RealmList},
            BnetRealmHandle,
        },
    },
};
use bnet_rpc::{
    bgs::protocol::{
        self,
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
use bytes::Bytes;
use prost::Message;
use rand::{rngs::OsRng, RngCore};
use sqlx::Row;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, trace, warn};

use crate::{rest::LoginRESTService, ssl_context::SslContext};

fn handle_game_utils_service_compress_response<T>(prefix: &[u8], data: &T) -> BnetRpcResult<Vec<u8>>
where
    T: ?Sized + serde::Serialize,
{
    let mut data = match serde_json::to_vec(&data) {
        Err(e) => {
            error!(target:"session", "unable to serialise {} to json, err={e}", String::from_utf8_lossy(prefix));
            return Err(BattlenetRpcErrorCode::UtilServerFailedToSerializeResponse);
        },
        Ok(r) => r,
    };
    let mut json = prefix.to_vec();
    json.append(&mut data);
    match bnetrpc_zcompress(json) {
        Err(e) => {
            error!(target:"session", cause=%e, "unable to compress {}", String::from_utf8_lossy(prefix));
            Err(BattlenetRpcErrorCode::UtilServerFailedToSerializeResponse)
        },
        Ok(c) => Ok(c),
    }
}

#[derive(Debug)]
struct AccountInfo {
    // void LoadResult(PreparedQueryResult result),
    id:                    u32,
    login:                 String,
    is_locked_to_ip:       bool,
    lock_country:          String,
    last_ip:               String,
    login_ticket_expiry:   u64,
    is_banned:             bool,
    is_permanently_banned: bool,
    game_accounts:         BTreeMap<u32, GameAccountInfo>,
}

impl AccountInfo {
    fn load_result<'r, R>(result: &'r [R]) -> Self
    where
        R: sqlx::Row,
        usize: sqlx::ColumnIndex<R>,
        String: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
        u8: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
        u32: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
        u64: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    {
        // ba.id, ba.email, ba.locked, ba.lock_country, ba.last_ip, ba.LoginTicketExpiry, bab.unbandate > UNIX_TIMESTAMP() OR bab.unbandate = bab.bandate, bab.unbandate = bab.bandate FROM battlenet_accounts ba LEFT JOIN battlenet_account_bans bab WHERE email = ?
        let fields = &result[0];
        let id = fields.get(0);
        let login = fields.get(1);
        let is_locked_to_ip = fields.get::<u8, _>(2) != 0;
        let lock_country = fields.get(3);
        let last_ip = fields.get(4);
        let login_ticket_expiry = fields.get(5);
        let is_banned = fields.get::<u64, _>(6) != 0;
        let is_permanently_banned = fields.get::<u64, _>(7) != 0;

        let mut game_accounts = BTreeMap::new();
        const GAME_ACCOUNT_FIELDS_OFFSET: usize = 8;

        for fields in result.iter() {
            game_accounts
                .entry(fields.get(GAME_ACCOUNT_FIELDS_OFFSET))
                .or_insert_with(|| GameAccountInfo::load_result::<'r, R>(fields, GAME_ACCOUNT_FIELDS_OFFSET));
        }

        Self {
            id,
            login,
            is_locked_to_ip,
            lock_country,
            last_ip,
            login_ticket_expiry,
            is_banned,
            is_permanently_banned,
            game_accounts,
        }
    }
}

#[derive(Debug, Clone)]
struct LastPlayedCharacterInfo {
    realm_id:         BnetRealmHandle,
    character_name:   String,
    character_guid:   u64,
    last_played_time: u32,
}

#[derive(Clone, Debug)]
struct GameAccountInfo {
    // void LoadResult(Field* fields);
    id:                     u32,
    name:                   String,
    display_name:           String,
    unban_date:             u64,
    is_banned:              bool,
    is_permanently_banned:  bool,
    _security_level:        AccountTypes,
    character_counts:       BTreeMap<u32, /*realmAddress*/ u8>,
    last_played_characters: BTreeMap<String /*: subRegion*/, LastPlayedCharacterInfo>,
}

impl GameAccountInfo {
    fn load_result<'r, R>(fields: &'r R, offset: usize) -> Self
    where
        R: sqlx::Row,
        usize: sqlx::ColumnIndex<R>,
        String: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
        u8: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
        u32: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
        u64: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    {
        // a.id, a.username, ab.unbandate, ab.unbandate = ab.bandate, aa.gmlevel
        let id = fields.get(offset);
        let name = fields.get::<String, _>(offset + 1);
        let unban_date = fields.get(offset + 2);
        let is_permanently_banned = fields.get::<u32, _>(offset + 3) != 0;
        let is_banned = is_permanently_banned || unban_date > unix_now().as_secs();
        let _security_level = fields.get::<u8, _>(offset + 4).try_into().unwrap_or(AccountTypes::SecPlayer);

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

pub struct Session {
    // Lifetime management
    base:       SharedFromSelfBase<Session>,
    task:       OnceLock<JoinHandle<AzResult<()>>>,
    rt_handler: tokio::runtime::Handle,

    // Rest of Session info.
    /// Contains a mapping of tokens expecting a return from a previous client
    /// call to service hash and method_id
    ///
    response_callbacks: Mutex<BTreeMap<u32, (u32, u32)>>,
    /// Replaces GetRemoteIpAddress in TC/AC
    ip_or_name:         AddressOrName,
    inner:              SocketWrappper,
    /// the current session's account information. If this is set, then we can treat it as _authed=True
    /// like in TC/AC
    account_info:       OnceLock<AccountInfo>,
    /// Game account info, saved during Command_RealmListTicketRequest_v1_b9
    /// for now its a cloned value from `account_info`
    ///  
    /// TODO: Should this be rwlock instead?
    game_account_info:  OnceLock<GameAccountInfo>,
    locale:             OnceLock<Locale>,
    os:                 OnceLock<String>,
    build:              AtomicU32,
    ip_country:         OnceLock<String>,
    client_secret:      OnceLock<[u8; 32]>,
    request_token:      AtomicU32,
}

fn map_err_to_denied<E: std::error::Error>(e: E) -> BattlenetRpcErrorCode {
    error!(target:"session::rpc", cause=%e, "error when making sql queries in bnet session. mapped to denied");
    BattlenetRpcErrorCode::Denied
}

fn map_err_to_internal<E: std::error::Error>(e: E) -> BattlenetRpcErrorCode {
    error!(target:"session::rpc", cause=%e, "error when making sql queries in bnet session. mapped to Internal");
    BattlenetRpcErrorCode::Internal
}

impl Session {
    async fn receive(&self, service_hash: u32, token: u32, method_id: u32, _packet_buffer: Bytes) -> io::Result<()> {
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
                debug!(target:"session::rpc", caller_info=self.caller_info(), "client tried to call invalid service {:?}", c);
            },
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, target="session", fields(caller_info=self.caller_info()))]
    async fn verify_web_credentials(&self, web_credentials: &[u8]) -> BnetRpcResult<NoData> {
        if web_credentials.is_empty() {
            return Err(BattlenetRpcErrorCode::Denied);
        }
        let login_db = LoginDatabase::get();
        let result = LoginDatabase::sel_bnet_account_info(login_db, params!(web_credentials))
            .await
            .map_err(map_err_to_denied)?;

        if result.is_empty() {
            return Err(BattlenetRpcErrorCode::Denied);
        }

        let mut account_info = AccountInfo::load_result(&result);
        if account_info.login_ticket_expiry < unix_now().as_secs() {
            return Err(BattlenetRpcErrorCode::TimedOut);
        }
        let character_counts_result = LoginDatabase::sel_bnet_character_counts_by_bnet_id(login_db, params!(account_info.id))
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
        let last_player_characters_result = LoginDatabase::sel_bnet_last_player_characters(login_db, params!(account_info.id))
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
        let ip_address = self.ip_or_name.ip_str_or_name();

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
        if account_info.is_banned {
            if account_info.is_permanently_banned {
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

impl SharedFromSelf<Session> for Session {
    fn get_base(&self) -> &SharedFromSelfBase<Session> {
        &self.base
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // cancels the socket and unsets the read receiver
        _ = self.rt_handler.block_on(self.close());
        // Ensures that the write join handler is properly handled
        if let Some(jh) = self.task.take() {
            match self.rt_handler.block_on(jh) {
                Err(e) => {
                    error!(target:"session", cause=?e, "error joining on runtime when dropping socket");
                },
                Ok(Err(e)) => {
                    error!(target:"session", cause=?e, "cause of error when dropping session");
                },
                Ok(Ok(_)) => {},
            }
        }
    }
}

impl Socket for Session {
    async fn new_from_tcp(rt_handler: tokio::runtime::Handle, cancel_token: CancellationToken, addr: AddressOrName, tcp_conn: TcpStream) -> AzResult<Arc<Self>>
    where
        Self: std::marker::Sized,
    {
        let conn = SslContext::get().accept(tcp_conn).await?;
        let (rd, wr) = tokio::io::split(conn);
        Ok(Self::new(rt_handler, cancel_token, addr, Box::new(rd), Box::new(wr)))
    }

    fn new<R, W>(rt_handler: tokio::runtime::Handle, cancel_token: CancellationToken, name: AddressOrName, rd: R, wr: W) -> Arc<Self>
    where
        R: AsyncRead + Unpin + Send + Sync + 'static,
        W: AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let inner = SocketWrappper::new(rt_handler.clone(), cancel_token, name.clone(), rd, wr);
        let s = Arc::new(Self {
            base: SharedFromSelfBase::new(),
            task: OnceLock::new(),
            response_callbacks: Mutex::new(BTreeMap::new()),
            ip_or_name: name,
            inner,
            rt_handler,
            account_info: OnceLock::new(),
            game_account_info: OnceLock::new(),
            build: AtomicU32::new(0),
            locale: OnceLock::new(),
            os: OnceLock::new(),
            client_secret: OnceLock::new(),
            ip_country: OnceLock::new(),
            request_token: AtomicU32::new(0),
        });
        // accountinfo
        s.base.initialise(&s);
        s
    }

    async fn start(&self) -> AzResult<()> {
        // CheckIpCallback routine
        let ip_address = self.ip_or_name.ip_str_or_name();
        trace!(target:"session", caller = self.caller_info(), "Accepted connection");

        // Verify that this IP is not in the ip_banned table
        let login_db = LoginDatabase::get();
        LoginDatabase::del_expired_ip_bans(login_db, params!()).await?;
        for fields in LoginDatabase::sel_ip_info(login_db, params!(ip_address)).await? {
            let banned = fields.get::<u64, _>("banned") != 0;
            if banned {
                debug!(target:"session", "[CheckIpCallback] Banned ip '{}' tries to login!", self.ip_or_name);
                return self.close().await;
            }
        }

        let this = self.shared_from_self();
        // begin AsyncRead => ReadHandler routine
        self.task.get_or_init(move || {
            this.clone().rt_handler.spawn(async move {
                loop {
                    this.is_running()?;

                    // ReadHeaderLengthHandler
                    let header_length = u16::from_be_bytes(this.inner.receive(2).await?.to_vec().try_into().unwrap());
                    // ReadHeaderHandler
                    let header = Header::decode(this.inner.receive(header_length.into()).await?)?;
                    // ReadDataHandler
                    let packet_buffer = this.inner.receive(header.size().try_into().unwrap()).await?;
                    if header.service_id != 0xFE {
                        this.dispatch(header.service_hash(), header.token, header.method_id(), packet_buffer).await?;
                    } else {
                        this.receive(header.service_hash(), header.token, header.method_id(), packet_buffer).await?;
                    }
                }
            })
        });

        Ok(())
    }

    fn is_running(&self) -> AzResult<()> {
        self.inner.is_running().map_err(AzError::new)
    }

    async fn close(&self) -> AzResult<()> {
        self.inner.close_socket().await.map_err(AzError::new)
    }
}

impl BnetRpcService for Session {
    fn caller_info(&self) -> String {
        let mut stream = format!("[{}", self.ip_or_name);
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

    async fn send_server_response<M>(&self, response: bnet_rpc::BnetServiceWrapper<M>) -> std::io::Result<()>
    where
        M: prost::Message,
    {
        let BnetServiceWrapper { token, result, .. } = response;

        let (status, response) = match result {
            Err(e) => (Some(e.into()), None),
            Ok(p) => (None, Some(p)),
        };

        let header = protocol::Header {
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
        self.inner.write(packet).await?;

        Ok(())
    }

    async fn make_client_request<M>(&self, request: bnet_rpc::BnetServiceWrapper<M>) -> std::io::Result<()>
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

        let header = protocol::Header {
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
        self.inner.write(packet).await?;

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

impl AuthenticationListener for Session {
    const USE_ORIGINAL_HASH: bool = true;
}

impl ChallengeListener for Session {
    const USE_ORIGINAL_HASH: bool = true;
}

impl AccountService for Session {
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
                gs.suspension_expires = Some(ga.unban_date * 1000000);
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

impl AuthenticationService for Session {
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

        // self.ip_or_name
        let endpoint = LoginRESTService::get().get_address_for_client(&self.ip_or_name);
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

impl ChannelService for Session {
    const USE_ORIGINAL_HASH: bool = true;
}

impl ConnectionService for Session {
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

        let this = self.shared_from_self();
        self.rt_handler.spawn(async move {
            let err = this.close().await;
            trace!(target:"session", cause=?err, "closed session due to request to be disconnected");
        });
        Ok(NoResponse {})
    }
}

impl FriendsService for Session {
    const USE_ORIGINAL_HASH: bool = true;
}

impl GameUtilitiesService for Session {
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
                attribute_value: RealmList::get()
                    .get_sub_regions()
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

impl PresenceService for Session {
    const USE_ORIGINAL_HASH: bool = true;
}

impl ReportService for Session {
    const USE_ORIGINAL_HASH: bool = true;
}

impl ResourcesService for Session {
    const USE_ORIGINAL_HASH: bool = true;
}

impl UserManagerService for Session {
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
struct RealmEntry {
    #[serde(rename = "wowRealmAddress")]
    wow_realm_address: u32,
    #[serde(rename = "cfgTimezonesID")]
    cfg_timezones_id:  u32,
    #[serde(rename = "populationState")]
    population_state:  u32,
    #[serde(rename = "cfgCategoriesID")]
    cfg_categories_id: u32,
    version:           ClientVersion,
    #[serde(rename = "cfgRealmsID")]
    cfg_realms_id:     u32,
    flags:             u32,
    name:              String,
    #[serde(rename = "cfgConfigsID")]
    cfg_configs_id:    u32,
    #[serde(rename = "cfgLanguagesID")]
    cfg_languages_id:  u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct RealmState {
    update:   Option<RealmEntry>,
    deleting: bool,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct RealmListUpdates {
    updates: Vec<RealmState>,
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

impl Session {
    async fn get_realm_list_ticket(&self, params: BTreeMap<String, Variant>) -> BnetRpcResult<ClientResponse> {
        let Some(accnt_info) = self.account_info.get() else {
            return Err(BattlenetRpcErrorCode::WowServicesDeniedRealmListTicket);
        };
        if let Some(identity) = params.get("Param_Identity") {
            let json_start = identity.blob_value();
            match serde_json::from_slice(json_start) {
                Err(e) => {
                    warn!(target:"session", cause=%e, identity_val=String::from_utf8_lossy(json_start).to_string(), current=?accnt_info, "error serialising params identity from json");
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
            let json_start = client_info.blob_value();
            match serde_json::from_slice(json_start) {
                Err(e) => {
                    warn!(target:"session", cause=%e, val=String::from_utf8_lossy(json_start).to_string(), current=?accnt_info, "error serialising params identity from json");
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

        let login_db = LoginDatabase::get();
        LoginDatabase::upd_bnet_last_login_info(
            login_db,
            params!(
                self.ip_or_name.ip_str_or_name(),
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
            let Some(realm_entry) = RealmList::get().get_realm_entry_json(&last_player_char.realm_id, self.build.load(Ordering::SeqCst)) else {
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

        let realm_list = RealmList::get().get_realm_list(self.build.load(Ordering::SeqCst), &sub_region_id);
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
        let server_addresses = RealmList::get()
            .retrieve_realm_list_server_ip_addresses(realm_address.uint_value() as u32, &self.ip_or_name, self.build.load(Ordering::SeqCst))
            .map_err(map_join_realm_err)?;
        let server_addresses = handle_game_utils_service_compress_response(b"JSONRealmListServerIPAddresses:", &server_addresses)?;
        let server_secret = RealmList::get()
            .join_realm(
                &self.ip_or_name,
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
