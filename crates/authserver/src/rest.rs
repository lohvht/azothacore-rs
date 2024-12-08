use std::sync::Arc;

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
    bevy_app::{az_startup_succeeded, AzStartupFailedEvent, TokioRuntime},
    configuration::ConfigMgr,
    hex_str,
    utils::unix_now,
};
use azothacore_database::{
    args_unwrap,
    database_env::{LoginDatabase, LoginPreparedStmts},
};
use azothacore_server::{game::accounts::battlenet_account_mgr::BattlenetAccountMgr, shared::networking::socket::AddressOrName};
use bevy::{
    app::AppExit,
    prelude::{App, Commands, EventReader, EventWriter, IntoSystemConfigs, PostUpdate, Res, Resource, Startup, SystemSet},
};
use hyper::{body::Incoming, service::service_fn, StatusCode};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder as HyperServerConnBuilder,
};
use rand::{rngs::OsRng, Rng};
use sqlx::Row;
use tokio::{
    net::TcpListener,
    sync::mpsc::{unbounded_channel, UnboundedSender},
};
use tower_service::Service as TowerService;
use tracing::{debug, error, info, warn};

use crate::{
    config::{AuthserverConfig, WrongPassBanType},
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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoginRESTServiceSystemSets {
    Start,
    Terminate,
}

pub fn login_rest_service_plugin(app: &mut App) {
    app.add_systems(Startup, LoginRESTService::start.in_set(LoginRESTServiceSystemSets::Start))
        .add_systems(
            PostUpdate,
            LoginRESTService::terminate
                .run_if(az_startup_succeeded())
                .in_set(LoginRESTServiceSystemSets::Terminate),
        );
}

#[derive(Resource)]
struct LoginRestTermSender(UnboundedSender<()>);

struct LoginRESTService;

impl LoginRESTService {
    fn start(
        mut commands: Commands,
        cfg: Res<ConfigMgr<AuthserverConfig>>,
        rt: Res<TokioRuntime>,
        ssl_ctx: Res<SslContext>,
        login_db: Res<LoginDatabase>,
        mut ev_startup_failed: EventWriter<AzStartupFailedEvent>,
    ) {
        let (term_snd, mut term_rcv) = unbounded_channel();
        commands.insert_resource(LoginRestTermSender(term_snd));
        let acceptor = match rt.block_on(TcpListener::bind(cfg.login_rest_bind_addr())) {
            Ok(a) => a,
            Err(e) => {
                error!(target:"server::rest", cause=%e, "Couldn't bind to {}", cfg.login_rest_bind_addr());
                ev_startup_failed.send_default();
                return;
            },
        };
        info!(target:"server::rest", "Login service bound to http://{}", cfg.login_rest_bind_addr());

        let cfg = Arc::new((**cfg).clone());
        let ssl_ctx = ssl_ctx.clone();
        let login_db = Arc::new(login_db.clone());

        let handler = rt.handle().clone();
        rt.spawn(async move {
            let router = Router::new()
                .route("/bnetserver/login/", get(Self::handle_get_form))
                .route("/bnetserver/gameAccounts/", get(Self::handle_get_game_accounts))
                .route("/bnetserver/portal/", get(Self::handle_get_portal))
                .route("/bnetserver/login/", post(Self::handle_post_login))
                .route("/bnetserver/refreshLoginTicket/", post(Self::handle_post_refresh_login_ticket))
                .fallback(Self::handle_404);

            loop {
                let (cnx, remote_addr) = tokio::select! {
                    _ = term_rcv.recv() => {
                        debug!("termination triggered, quitting login rest service loop");
                        break
                    }
                    // Wait for new tcp connection
                    accepted = acceptor.accept() => match accepted {
                        Ok(a) => a,
                        Err(e) => {
                            error!(target:"server::rest", "error encountered when accepting request: {e}");
                            continue;
                        },
                    }
                };

                debug!(target:"server::rest", "Accepted connection from Addr {remote_addr}");

                // Wait for tls handshake to happen
                let stream = match ssl_ctx.accept(cnx).await {
                    Ok(s) => s,
                    Err(e) => {
                        error!(target:"server::rest", "Failed SSL handshake from Addr {remote_addr}, err: {e}");
                        continue;
                    },
                };

                // Hyper has its own `AsyncRead` and `AsyncWrite` traits and doesn't use tokio.
                // `TokioIo` converts between them.
                let stream = TokioIo::new(stream);
                let state = LoginServiceRequestState {
                    source_ip: remote_addr.into(),
                    login_db:  login_db.clone(),
                    cfg:       cfg.clone(),
                };

                let router = router.clone();
                // Hyper has also its own `Service` trait and doesn't use tower. We can use
                // `hyper::service::service_fn` to create a hyper `Service` that calls our app through
                // `tower::Service::call`.
                let hyper_svc = service_fn(move |request: Request<Incoming>| {
                    // We have to clone `tower_svc` because hyper's `Service` uses `&self` whereas
                    // tower's `Service` requires `&mut self`.
                    //
                    // We don't need to call `poll_ready` since `Router` is always ready.
                    router.clone().with_state(state.clone()).call(request)
                });

                handler.spawn(async {
                    let mut builder = HyperServerConnBuilder::new(TokioExecutor::new());
                    builder.http1().title_case_headers(true);

                    let _ = builder
                        // .serve_connection(stream, hyper_svc)
                        .serve_connection_with_upgrades(stream, hyper_svc)
                        .await;
                });
            }
            info!(target:"server::rest", "Login service exiting...");
        });
    }

    fn terminate(mut app_exit_events: EventReader<AppExit>, term_snds: Res<LoginRestTermSender>) {
        let mut sent_exit = false;
        for _ev in app_exit_events.read() {
            if !sent_exit {
                // NOTE: run asynchronously w/out needing for error handling (For now)
                // this should be short so it seems fairly okay to do
                //
                // This is just an attempt to terminate the accept loop, the program
                // may go ahead and exit anyway via a tokio runtime cancellation.
                if let Err(e) = term_snds.0.send(()) {
                    debug!(cause=?e, "send terminate error, terminate network receiving channel half may be dropped or closed");
                }
                sent_exit = true;
            }
            // We still wanna at least process the rest of the exits anyway, if any.
            // so no break.
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
        State(LoginServiceRequestState { login_db, .. }): State<LoginServiceRequestState>,
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

        let result = handle_resp_error!(
            LoginDatabase::sel_bnet_game_account_list(&**login_db, args_unwrap!(basic_auth.username())).await,
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

    async fn handle_get_portal(State(LoginServiceRequestState { source_ip, cfg, .. }): State<LoginServiceRequestState>) -> String {
        let endpoint = cfg.login_rest_get_address_for_client(&source_ip);
        endpoint.to_string()
    }

    async fn handle_post_login(
        State(LoginServiceRequestState { login_db, source_ip, cfg, .. }): State<LoginServiceRequestState>,
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

        let fields = match handle_login_err!(
            LoginDatabase::sel_bnet_authentication(&**login_db, args_unwrap!(&login)).await,
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
        let is_banned = is_banned.is_some_and(|b| b);

        let now = unix_now().as_secs();
        if sent_password_hash == pass_hash {
            if login_ticket.is_none() || login_ticket_expiry.is_none_or(|exp_ts| exp_ts < now) {
                login_ticket = Some(format!("AZ-{}", hex_str!(OsRng.gen::<[u8; 20]>())));
            }
            let new_expiry = now + cfg.LoginREST.TicketDuration.as_secs();
            let res = LoginDatabase::upd_bnet_authentication(&**login_db, args_unwrap!(&login_ticket, new_expiry, account_id)).await;
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
            if !cfg.WrongPass.Enabled {
                return (StatusCode::OK, [("Content-Type", "application/json;charset=utf-8")], Json(error_response));
            }
            if !cfg.WrongPass.Logging {
                warn!(target:"server::rest", ip_address=%&source_ip, login=login, account_id=account_id, "Attempted to connect with wrong password!");
            }
            let mut trans = handle_login_err!(login_db.begin().await, "unable to open a transaction to update wrong password counts");
            handle_login_err!(
                LoginDatabase::upd_bnet_failed_logins(&mut *trans, args_unwrap!(account_id)).await,
                "unable to update bnet failed logins"
            );

            failed_logins += 1;
            debug!(target:"server::rest", MaxWrongPass=cfg.WrongPass.MaxCount,  account_id=account_id);
            if failed_logins < cfg.WrongPass.MaxCount {
                return (StatusCode::OK, [("Content-Type", "application/json;charset=utf-8")], Json(error_response));
            }
            let ban_time = cfg.WrongPass.BanTime.as_secs();
            if matches!(cfg.WrongPass.BanType, WrongPassBanType::BanAccount) {
                handle_login_err!(
                    LoginDatabase::ins_bnet_account_auto_banned(&mut *trans, args_unwrap!(account_id, ban_time)).await,
                    "unable to insert bnet auto ban"
                );
            } else {
                handle_login_err!(
                    LoginDatabase::ins_ip_auto_banned(&mut *trans, args_unwrap!(source_ip.to_string(), ban_time)).await,
                    "unable to insert IP ban"
                );
            }
            handle_login_err!(
                LoginDatabase::upd_bnet_reset_failed_logins(&mut *trans, args_unwrap!(account_id)).await,
                "unable to reset account failed logins"
            );

            handle_login_err!(trans.commit().await, "error commiting failed login update");
        }

        (StatusCode::OK, [("Content-Type", "application/json;charset=utf-8")], Json(error_response))
    }

    async fn handle_post_refresh_login_ticket(
        TypedHeader(basic_auth): TypedHeader<Authorization<Basic>>,
        State(LoginServiceRequestState { login_db, cfg, .. }): State<LoginServiceRequestState>,
    ) -> WrappedResponseResult<Json<LoginRefreshResult>, ErrorEmptyResponse> {
        if basic_auth.username().is_empty() {
            return err_empty_resp(StatusCode::UNAUTHORIZED);
        }

        let mut login_refresh_result = LoginRefreshResult::default();
        let login_ticket_expiry = match LoginDatabase::sel_bnet_existing_authentication(&**login_db, args_unwrap!(basic_auth.username())).await {
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

        let now = unix_now().as_secs();
        if login_ticket_expiry > now {
            let new_expiry = now + cfg.LoginREST.TicketDuration.as_secs();
            match LoginDatabase::upd_bnet_existing_authentication(&**login_db, args_unwrap!(new_expiry, basic_auth.username())).await {
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
    login_db:  Arc<LoginDatabase>,
    cfg:       Arc<AuthserverConfig>,
}
