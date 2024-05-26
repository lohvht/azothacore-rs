use std::{net::IpAddr, path::PathBuf, time::Duration};

use azothacore_common::{
    bounded_nums::LowerBoundedNum,
    configuration::{DatabaseInfo, DbUpdates, LogAppender, LogFlags, LogLevel, LogLoggerConfig},
    durationb_hours,
    durationb_mins,
    durationb_s,
};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum WrongPassBanType {
    BanIP,
    BanAccount,
}

structstruck::strike! {
#[strikethrough[serde_inline_default]]
#[strikethrough[derive(DefaultFromSerde, Deserialize, Serialize, Clone, Debug, PartialEq)]]
#[strikethrough[expect(non_snake_case)]]
pub struct AuthserverConfig {
    /// Database connection settings for the realm server.
    #[serde_inline_default(DatabaseInfo::default_with_info("azcore_auth"))] pub LoginDatabaseInfo: DatabaseInfo,
    /// Database Update settings
    #[serde_inline_default(DbUpdates{EnableDatabases: None.into(), ..Default::default() })] pub Updates: DbUpdates,
    // /// MaxPingTime in TC/AC
    // #[serde(default)] pub DBPingInterval: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(30) }>,
    /// Logs directory path - all logs will be written inside this directory.
    #[serde_inline_default("logs".into())] pub LogsDir: PathBuf,
    /// Auth server PID file.
    #[serde(default)] pub PidFile: Option<PathBuf>,
    #[serde(default="default_authserver_log_appenders")] pub Appender: Vec<LogAppender>,
    #[serde(default="default_authserver_log_configs")] pub Logger: Vec<LogLoggerConfig>,
    /// Bind auth server to IP/hostname
    #[serde_inline_default("0.0.0.0".parse().unwrap())] pub BindIP: IpAddr,
    /// TCP port to reach the auth server for battle.net connections.
    #[serde_inline_default(1119)] pub BattlenetPort: u16,
    /// Login REST service - this is used by the client to log in.
    #[serde(default)] pub LoginREST: pub struct AuthserverConfigLoginREST {
        /// TCP port to reach the REST login method.
        #[serde_inline_default(8081)] pub Port: u16,
        /// IP address sent to clients connecting from outside the network where bnetserver runs
        ///
        /// Set it to your external IP address
        #[serde_inline_default("127.0.0.1".parse().unwrap())] pub ExternalAddress: IpAddr,
        /// IP address sent to clients connecting from inside the network where bnetserver runs
        ///
        /// Set it to your local IP address (common 192.168.x.x network)
        ///
        /// or leave it at default value 127.0.0.1 if connecting directly to the internet without a router
        #[serde_inline_default("127.0.0.1".parse().unwrap())] pub LocalAddress: IpAddr,
        /// Subnet mask for local network address
        ///
        /// Set it to your local IP address netmask or leave it as its default at 255.255.255.0
        #[serde_inline_default("255.255.255.0".parse().unwrap())] pub SubnetMask: IpAddr,
        /// Determines how long the login ticket is valid
        ///
        /// When using client -launcherlogin feature it is recommended to set it to a high value (like a week)
        #[serde(default)] pub TicketDuration: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_hours!(15) }>,
    },
    #[serde(default)] pub WrongPass: pub struct AuthserverConfigWrongPass {
        #[serde(default)] pub Enabled: bool,
        /// Number of login attempts with wrong password before the account or IP will be banned.
        #[serde_inline_default(5)] pub MaxCount: u64,
        /// Time for banning account or IP for invalid login attempts.
        #[serde(default)] pub BanTime: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(10) }>,
        /// Ban type for invalid login attempts - can ban by IP or by Account
        #[serde_inline_default(WrongPassBanType::BanIP)] pub BanType: WrongPassBanType,
        /// log attempted wrong password
        #[serde_inline_default(false)] pub Logging: bool,
    },
    /// Certificates file - this file is used by both the Auth bnet server as well as the client
    /// to ensure that TLS is established between the both of them
    ///
    /// THE CLIENT NEEDS TO BE PATCHED WITH THE SAME CERTS USED IN THE SERVER!!!
    #[serde_inline_default("bnetserver.cert.pem".into())] pub CertificatesFile: PathBuf,
    /// Private key file - this file is used by both the Auth bnet server as well as the client
    /// to ensure that TLS is established between the both of them.
    ///
    /// THE CLIENT NEEDS TO BE PATCHED WITH THE SAME CERTS USED IN THE SERVER!!!
    // TODO: hirogoro@26may2024: implement connection patcher for the client exe.
    #[serde_inline_default("bnetserver.key.pem".into())] pub PrivateKeyFile: PathBuf,
    /// Time between realm list updates.
    #[serde(default)] pub RealmsStateUpdateDelay: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_s!(10) }>,
    /// Time between checks for expired bans
    #[serde(default)] pub BanExpiryCheckInterval: LowerBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(1) }>,
}
}

pub fn default_authserver_log_appenders() -> Vec<LogAppender> {
    // use LogConsoleColours::*;
    use LogFlags::*;
    use LogLevel::*;
    vec![
        LogAppender::Console {
            name:      String::from("Console"),
            min_level: Info,
            max_level: Error,
            flags:     AddLogLevel | AddLogFilter | TruncateFile | BackupBeforeOverwrite,
            // colours: vec![
            //     (Fatal, Red),
            //     (Error, Lred),
            //     (Warning, Brown),
            //     (Info, Green),
            //     (Debug, Cyan),
            //     (Trace, Magenta),
            // ],
        },
        LogAppender::File {
            name:      String::from("Auth"),
            min_level: Warning,
            max_level: Error,
            flags:     AddLogLevel | AddLogFilter | TruncateFile | BackupBeforeOverwrite | AddLogTimestamps,
            file:      String::from("Auth.log"),
        },
    ]
}

pub fn default_authserver_log_configs() -> Vec<LogLoggerConfig> {
    use LogLevel::*;
    vec![LogLoggerConfig {
        name:      String::from("root"),
        min_level: Info,
        max_level: Error,
        appenders: vec![String::from("Console"), String::from("Auth")],
    }]
}
