use std::{
    net::{IpAddr, SocketAddr},
    sync::OnceLock,
    time::Duration,
};

use axum::{
    extract::{rejection::JsonRejection, State},
    http::Request,
    response::IntoResponse,
    routing::{get, post},
    Json,
    Router,
};
use axum_extra::{
    extract::WithRejection,
    headers::{authorization::Basic, Authorization},
    TypedHeader,
};
use azothacore_common::{
    az_error,
    hex_str,
    r#async::Context,
    utils::{net_resolve, unix_now},
    AzResult,
};
use azothacore_database::{
    database_env::{LoginDatabase, LoginPreparedStmts},
    params,
};
use azothacore_server::{game::accounts::battlenet_account_mgr::BattlenetAccountMgr, shared::networking::socket::AddressOrName};
use hyper::{body::Incoming, service::service_fn, StatusCode};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder as HyperServerConnBuilder,
};
use ipnet::IpNet;
use rand::{rngs::OsRng, Rng};
use sqlx::Row;
use tokio::net::{TcpListener, TcpStream};
use tower_service::Service as TowerService;
use tracing::{debug, error, info, warn};

use crate::{
    config::{AuthserverConfigLoginREST, AuthserverConfigWrongPass, WrongPassBanType},
    ssl_context::SslContext,
};

struct WrappedResponseResult<T, E>(Result<T, E>);

impl<T, E> ::std::ops::Deref for WrappedResponseResult<T, E> {
    type Target = Result<T, E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, E> ::std::ops::DerefMut for WrappedResponseResult<T, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, E> From<Result<T, E>> for WrappedResponseResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        WrappedResponseResult(value)
    }
}

impl<T, E> IntoResponse for WrappedResponseResult<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> axum::response::Response {
        match self.0 {
            Ok(o) => o.into_response(),
            Err(e) => e.into_response(),
        }
    }
}

pub struct LoginRESTService;

struct LoginServiceDetails {
    bind_addr:             SocketAddr,
    local_address:         SocketAddr,
    local_network:         IpNet,
    external_address:      SocketAddr,
    login_ticket_duration: Duration,
    wrong_pass:            AuthserverConfigWrongPass,
}

impl LoginRESTService {
    pub async fn start(ctx: Context, bind_ip: IpAddr, login_rest: AuthserverConfigLoginREST, wrong_pass: AuthserverConfigWrongPass) -> AzResult<()> {
        let login_service_details = {
            let external_address = net_resolve((login_rest.ExternalAddress, login_rest.Port)).map_err(|e| {
                error!(target:"server::rest", cause=%e, "Could not resolve LoginREST.ExternalAddress {}", login_rest.ExternalAddress);
                e
            })?;
            let local_address = net_resolve((login_rest.LocalAddress, login_rest.Port)).map_err(|e| {
                error!(target:"server::rest", cause=%e, "Could not resolve LoginREST.LocalAddress {}", login_rest.LocalAddress);
                e
            })?;
            let bind_addr = net_resolve((bind_ip, login_rest.Port)).map_err(|e| {
                error!(target:"server::rest", cause=%e, "Could not resolve LoginREST.BindAddr {}", bind_ip);
                e
            })?;
            let local_subnet_mask = net_resolve((login_rest.SubnetMask, login_rest.Port)).map_err(|e| {
                error!(target:"server::rest", cause=%e, "Could not resolve LoginREST.SubnetMask {}", login_rest.SubnetMask);
                e
            })?;
            let local_network = IpNet::with_netmask(local_address.ip(), local_subnet_mask.ip())?;

            LOGIN_SERVICE_DETAILS.get_or_init(|| LoginServiceDetails {
                external_address,
                bind_addr,
                local_address,
                local_network,
                wrong_pass,
                login_ticket_duration: *login_rest.TicketDuration,
            })
        };

        let acceptor = TcpListener::bind(&login_service_details.bind_addr).await.map_err(|e| {
            error!(target:"server::rest", "Couldn't bind to {}", login_service_details.bind_addr);
            e
        })?;
        info!(target:"server::rest", "Login service bound to http://{}", login_service_details.bind_addr);

        let router = Router::new()
            .route("/bnetserver/login/", get(Self::handle_get_form))
            .route("/bnetserver/gameAccounts/", get(Self::handle_get_game_accounts))
            .route("/bnetserver/portal/", get(Self::handle_get_portal))
            .route("/bnetserver/login/", post(Self::handle_post_login))
            .route("/bnetserver/refreshLoginTicket/", post(Self::handle_post_refresh_login_ticket))
            .fallback(Self::handle_404);

        loop {
            let (cnx, addr) = tokio::select! {
                _ = ctx.cancelled() => {
                    break;
                },
                // Wait for new tcp connection
                a = acceptor.accept() => {
                    match a {
                        Ok(r) => r,
                        Err(e) => {
                            error!(target:"server::rest", "error encountered when accepting request: {e}");
                            continue;
                        }
                    }
                },
            };

            let tower_svc = router.clone().with_state(LoginServiceRequestState { source_ip: addr.into() });

            // NOTE: Unhandled for now, if theres an error just let it pass.
            ctx.spawn(serve_https_call(tower_svc, cnx, addr));
        }
        info!(target:"server::rest", "Login service exiting...");

        Ok(())
    }

    fn get_details() -> &'static LoginServiceDetails {
        LOGIN_SERVICE_DETAILS.get().expect("Expect login details to be set on startup at least")
    }

    pub fn get_address_for_client<'a>(address: &AddressOrName) -> &'a SocketAddr {
        let client_address = match address {
            // If its a name, we use local address
            AddressOrName::Name(_) => return &Self::get_details().local_address,
            AddressOrName::Addr(a) if a.ip().is_loopback() => return &Self::get_details().local_address,
            AddressOrName::Addr(a) => a,
        };
        if Self::get_details().local_address.ip().is_loopback() {
            return &Self::get_details().external_address;
        }

        if Self::get_details().local_network.contains(&client_address.ip()) {
            &Self::get_details().local_address
        } else {
            &Self::get_details().external_address
        }
    }

    async fn handle_404() -> impl IntoResponse {
        (StatusCode::NOT_FOUND, Json(Empty {}))
    }

    async fn handle_get_form() -> impl IntoResponse {
        let j = FormInputs {
            r#type: FormType::LOGIN_FORM,
            inputs: vec![
                FormInput {
                    input_id:   "account_name".to_string(),
                    r#type:     "text".to_string(),
                    label:      "E-mail".to_string(),
                    max_length: Some(320),
                },
                FormInput {
                    input_id:   "password".to_string(),
                    r#type:     "password".to_string(),
                    label:      "Password".to_string(),
                    max_length: Some(16),
                },
                FormInput {
                    input_id:   "log_in_submit".to_string(),
                    r#type:     "submit".to_string(),
                    label:      "Log In".to_string(),
                    max_length: None,
                },
            ],
        };
        (StatusCode::OK, [("Content-Type", "application/json;charset=utf-8")], Json(j))
    }

    async fn handle_get_game_accounts(
        TypedHeader(basic_auth): TypedHeader<Authorization<Basic>>,
    ) -> WrappedResponseResult<Json<GameAccountList>, ErrorEmptyResponse> {
        if basic_auth.username().is_empty() {
            return err_empty_resp(StatusCode::UNAUTHORIZED);
        }

        macro_rules! handle_resp_error {
            ( $err:expr, $status_code:expr ) => {{
                match $err {
                    Err(e) => {
                        tracing::error!(target:"server::rest", "server encounted error: err={e:?}");
                        return Err(($status_code, Json(Empty {}))).into();
                    },
                    Ok(o) => o
                }
            }};
        }

        let login_db = &LoginDatabase::get();
        let result = handle_resp_error!(
            LoginDatabase::sel_bnet_game_account_list(login_db, params!(basic_auth.username())).await,
            StatusCode::INTERNAL_SERVER_ERROR
        );

        fn format_display_name(name: String) -> String {
            if let Some(s) = name.find('#') {
                format!("WoW{}", &name[s + 1..])
            } else {
                name
            }
        }
        let mut response = GameAccountList {
            game_accounts: Vec::with_capacity(result.len()),
        };
        let now = unix_now();
        for fields in result {
            let display_name = format_display_name(fields.get(0));
            let expansion = fields.get(1);
            let ban_date: Option<u64> = fields.get(3);
            let unban_date: Option<u64> = fields.get(3);
            let suspension_reason = fields.get(4);

            let is_suspended = unban_date.map(|ud| ud > now.as_secs());
            let is_banned = match (ban_date, unban_date) {
                (Some(bd), Some(ud)) => Some(bd == ud),
                _ => None,
            };

            response.game_accounts.push(GameAccountInfo {
                display_name,
                expansion,
                is_suspended,
                is_banned,
                suspension_expires: unban_date,
                suspension_reason,
            });
        }
        Ok(Json(response)).into()
    }

    async fn handle_get_portal(State(state): State<LoginServiceRequestState>) -> String {
        let endpoint = Self::get_address_for_client(&state.source_ip);
        endpoint.to_string()
    }

    async fn handle_post_login(
        State(state): State<LoginServiceRequestState>,
        WithRejection(Json(login_form), _): WithRejection<Json<LoginForm>, PostLoginError>,
    ) -> impl IntoResponse {
        // following similar to TC's logic
        let error_response = LoginResult {
            authentication_state: AuthenticationState::DONE,
            error_code:           None,
            error_message:        None,
            login_ticket:         None,
            url:                  None,
        };
        macro_rules! handle_login_err {
            ( $res:expr, $msg:expr ) => {{
                match $res {
                    Err(e) => {
                        tracing::error!(target:"server::rest", "{}: err={e:?}", $msg);
                        return (StatusCode::OK,  [("Content-Type", "application/json;charset=utf-8")],Json(error_response));
                    },
                    Ok(o) => o
                }
            }};
        }

        let details = Self::get_details();

        let mut login = None;
        let mut password = None;
        for input in login_form.inputs {
            if login.is_none() && input.input_id == "account_name" {
                login = Some(input.value)
            } else if password.is_none() && input.input_id == "password" {
                password = Some(input.value)
            }
        }
        let (login, password) = match (login, password) {
            (Some(l), Some(p)) => (l.to_ascii_uppercase(), p.to_ascii_uppercase()),
            _ => {
                error!(target:"server::rest", "no login details found in request");
                return (
                    StatusCode::UNAUTHORIZED,
                    [("Content-Type", "application/json;charset=utf-8")],
                    Json(LoginResult {
                        authentication_state: AuthenticationState::LOGIN,
                        error_code:           Some("UNAUTHORIZED".to_string()),
                        error_message:        Some("There was an error while connecting to Battle.net due to wrong credentials".to_string()),
                        login_ticket:         None,
                        url:                  None,
                    }),
                );
            },
        };

        #[derive(sqlx::FromRow)]
        struct BnetAuth {
            account_id:          u32,
            pass_hash:           String,
            failed_logins:       u64,
            login_ticket:        Option<String>,
            login_ticket_expiry: Option<u64>,
            is_banned:           Option<bool>,
        }

        let login_db = &LoginDatabase::get();
        let fields = match handle_login_err!(
            LoginDatabase::sel_bnet_authentication(login_db, params!(&login)).await,
            "DB error for post login"
        ) {
            None => {
                debug!(target:"server::rest", "no login details found in DB");
                return (StatusCode::OK, [("Content-Type", "application/json;charset=utf-8")], Json(error_response));
            },
            Some(o) => o,
        };
        let sent_password_hash = BattlenetAccountMgr::calculate_sha_pass_hash(&login, &password);

        let BnetAuth {
            account_id,
            pass_hash,
            mut failed_logins,
            mut login_ticket,
            login_ticket_expiry,
            is_banned,
        } = fields;
        let is_banned = is_banned.map_or(false, |b| b);

        let now = unix_now().as_secs();
        if sent_password_hash == pass_hash {
            if login_ticket.is_none() || login_ticket_expiry.map_or(true, |exp_ts| exp_ts < now) {
                login_ticket = Some(format!("AZ-{}", hex_str!(OsRng.gen::<[u8; 20]>())));
            }
            let new_expiry = now + details.login_ticket_duration.as_secs();
            let res = LoginDatabase::upd_bnet_authentication(login_db, params!(&login_ticket, new_expiry, account_id)).await;
            if res.is_ok() {
                return (
                    StatusCode::OK,
                    [("Content-Type", "application/json;charset=utf-8")],
                    Json(LoginResult {
                        authentication_state: AuthenticationState::DONE,
                        login_ticket,
                        ..LoginResult::default()
                    }),
                );
            }
            warn!(target:"server::rest", "error somehow when calling DB to update bnet auth: err={res:?}");
        } else if !is_banned {
            let ip_address = &state.source_ip;
            if !details.wrong_pass.Enabled {
                return (StatusCode::OK, [("Content-Type", "application/json;charset=utf-8")], Json(error_response));
            }
            if !details.wrong_pass.Logging {
                warn!(target:"server::rest", ip_address=%ip_address, login=login, account_id=account_id, "Attempted to connect with wrong password!");
            }
            let mut trans = handle_login_err!(login_db.begin().await, "unable to open a transaction to update wrong password counts");
            handle_login_err!(
                LoginDatabase::upd_bnet_failed_logins(&mut *trans, params!(account_id)).await,
                "unable to update bnet failed logins"
            );

            failed_logins += 1;
            debug!(target:"server::rest", MaxWrongPass=details.wrong_pass.MaxCount,  account_id=account_id);
            if failed_logins < details.wrong_pass.MaxCount {
                return (StatusCode::OK, [("Content-Type", "application/json;charset=utf-8")], Json(error_response));
            }
            let ban_time = details.wrong_pass.BanTime.as_secs();
            if matches!(details.wrong_pass.BanType, WrongPassBanType::BanAccount) {
                handle_login_err!(
                    LoginDatabase::ins_bnet_account_auto_banned(&mut *trans, params!(account_id, ban_time)).await,
                    "unable to insert bnet auto ban"
                );
            } else {
                handle_login_err!(
                    LoginDatabase::ins_ip_auto_banned(&mut *trans, params!(ip_address.to_string(), ban_time)).await,
                    "unable to insert IP ban"
                );
            }
            handle_login_err!(
                LoginDatabase::upd_bnet_reset_failed_logins(&mut *trans, params!(account_id)).await,
                "unable to reset account failed logins"
            );

            handle_login_err!(trans.commit().await, "error commiting failed login update");
        }

        (StatusCode::OK, [("Content-Type", "application/json;charset=utf-8")], Json(error_response))
    }

    async fn handle_post_refresh_login_ticket(
        TypedHeader(basic_auth): TypedHeader<Authorization<Basic>>,
    ) -> WrappedResponseResult<Json<LoginRefreshResult>, ErrorEmptyResponse> {
        if basic_auth.username().is_empty() {
            return err_empty_resp(StatusCode::UNAUTHORIZED);
        }
        let mut login_refresh_result = LoginRefreshResult::default();
        let login_db = &LoginDatabase::get();
        let login_ticket_expiry = match LoginDatabase::sel_bnet_existing_authentication(login_db, params!(basic_auth.username())).await {
            Err(e) => {
                error!(target:"server::rest", username=basic_auth.username(), "unable to select existing bnet authentications; err={e}");
                login_refresh_result.is_expired = Some(true);
                return Ok(Json(login_refresh_result)).into();
            },
            Ok(None) => {
                error!(target:"server::rest", username=basic_auth.username(), "no existing bnet authentications");
                login_refresh_result.is_expired = Some(true);
                return Ok(Json(login_refresh_result)).into();
            },
            Ok(Some(fields)) => fields.get::<u64, _>(0),
        };

        let details = Self::get_details();
        let now = unix_now().as_secs();
        if login_ticket_expiry > now {
            let new_expiry = now + details.login_ticket_duration.as_secs();
            match LoginDatabase::upd_bnet_existing_authentication(login_db, params!(new_expiry, basic_auth.username())).await {
                Err(e) => {
                    error!(target:"server::rest", username=basic_auth.username(), "update bnet authentication failed: err={e}");
                    login_refresh_result.is_expired = Some(true);
                },
                Ok(_) => {
                    login_refresh_result.login_ticket_expiry = new_expiry;
                },
            };
        } else {
            login_refresh_result.is_expired = Some(true);
        }
        Ok(Json(login_refresh_result)).into()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct LoginRefreshResult {
    login_ticket_expiry: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_expired:          Option<bool>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum AuthenticationState {
    #[default]
    LOGIN = 1,
    LEGAL = 2,
    AUTHENTICATOR = 3,
    DONE = 4,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct LoginResult {
    authentication_state: AuthenticationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code:           Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message:        Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url:                  Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    login_ticket:         Option<String>,
}

// We derive `thiserror::Error`
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct PostLoginError(#[from] JsonRejection);

// We implement `IntoResponse` so PostLoginError can be used as a response
impl IntoResponse for PostLoginError {
    fn into_response(self) -> axum::response::Response {
        error!(target:"server::rest", "error deserialiing JSON for post login, err: {self}");
        let res = LoginResult {
            authentication_state: AuthenticationState::LOGIN,
            error_code:           Some("UNABLE_TO_DECODE".to_string()),
            error_message:        Some("There was an internal error while connecting to Battle.net. Please try again later.".to_string()),
            login_ticket:         None,
            url:                  None,
        };
        (StatusCode::BAD_REQUEST, Json(res)).into_response()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GameAccountInfo {
    display_name:       String,
    expansion:          u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_suspended:       Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_banned:          Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suspension_expires: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suspension_reason:  Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GameAccountList {
    game_accounts: Vec<GameAccountInfo>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FormInputValue {
    input_id: String,
    value:    String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LoginForm {
    platform_id: String,
    program_id:  String,
    version:     String,
    inputs:      Vec<FormInputValue>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[allow(non_camel_case_types)]
enum FormType {
    LOGIN_FORM = 1,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FormInputs {
    r#type: FormType,
    inputs: Vec<FormInput>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FormInput {
    input_id:   String,
    r#type:     String,
    label:      String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_length: Option<u32>,
}

type ErrorResponse<T> = (StatusCode, Json<T>);

type ErrorEmptyResponse = ErrorResponse<Empty>;

fn err_resp<T, U>(status: StatusCode, resp: U) -> WrappedResponseResult<T, ErrorResponse<U>> {
    Err((status, Json(resp))).into()
}

fn err_empty_resp<T>(status: StatusCode) -> WrappedResponseResult<T, ErrorEmptyResponse> {
    err_resp(status, Empty {})
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Empty {}

#[derive(Clone)]
struct LoginServiceRequestState {
    source_ip: AddressOrName,
}

async fn serve_https_call(router: Router<()>, stream: TcpStream, addr: SocketAddr) -> AzResult<()> {
    // Wait for tls handshake to happen
    let stream = match SslContext::get().accept(stream).await {
        Ok(s) => s,
        Err(e) => {
            error!(target:"server::rest", "Failed SSL handshake from Addr {addr}, err: {e}");
            return Err(e.into());
        },
    };

    debug!(target:"server::rest", "Accepted connection from Addr {addr}");

    // Hyper has its own `AsyncRead` and `AsyncWrite` traits and doesn't use tokio.
    // `TokioIo` converts between them.
    let stream = TokioIo::new(stream);

    // Hyper has also its own `Service` trait and doesn't use tower. We can use
    // `hyper::service::service_fn` to create a hyper `Service` that calls our app through
    // `tower::Service::call`.
    let hyper_svc = service_fn(move |request: Request<Incoming>| {
        // We have to clone `tower_svc` because hyper's `Service` uses `&self` whereas
        // tower's `Service` requires `&mut self`.
        //
        // We don't need to call `poll_ready` since `Router` is always ready.
        router.clone().call(request)
    });

    let mut builder = HyperServerConnBuilder::new(TokioExecutor::new());
    builder.http1().title_case_headers(true);

    builder
        // .serve_connection(stream, hyper_svc)
        .serve_connection_with_upgrades(stream, hyper_svc)
        .await
        .map_err(|e| {
            warn!(target:"server::rest", "error serving connection from {addr}: {e}");
            az_error!("{e}")
        })
}

static LOGIN_SERVICE_DETAILS: OnceLock<LoginServiceDetails> = OnceLock::new();
