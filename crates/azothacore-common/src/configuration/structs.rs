use std::{
    env::VarError,
    fmt::Debug,
    fs,
    hash::Hash,
    io,
    path::{Path, PathBuf},
};

use flagset::{flags, FlagSet};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;
use thiserror::Error;
use toml::{self, map::Map as TomlMap};
use tracing::error;

use crate::deref_boilerplate;

#[derive(Deserialize, Serialize)]
pub struct ConfigTable(TomlMap<String, toml::Value>);

deref_boilerplate!(ConfigTable, TomlMap<String, toml::Value>, 0);

#[derive(thiserror::Error, Debug)]
pub enum ConfigGetError {
    #[error("Key not found in config: '{keyname}'")]
    KeyNotFound { keyname: String },
    #[error(
        "Key references an inner toml table but value along the way is not a table: key: '{keyname}', tried_key_path: '{tried_key_path}', val found: {val}"
    )]
    KeyTableReference {
        keyname:        String,
        tried_key_path: String,
        val:            String,
    },
    #[error("error retrieving value from deserialised toml: {0}")]
    TomlDeserialisation(#[from] toml::de::Error),
    #[error("error retrieving from deserialised json: {0}")]
    JsonDeserialisation(#[from] serde_json::Error),
    #[error("error retrieving from environment variable: {0}")]
    EnvVar(#[from] VarError),
}

fn merge(first: &mut toml::Value, second: &mut toml::Value) {
    match [first, second] {
        [toml::Value::Table(first_t), toml::Value::Table(second_t)] => {
            second_t.iter_mut().for_each(|(k, second_v)| match first_t.get_mut(k) {
                None => {
                    // the first_table doesn't contain this key, we insert the whole second value in.
                    first_t.insert(k.clone(), second_v.clone());
                },
                Some(first_v) => {
                    merge(first_v, second_v);
                },
            })
        },
        [first_v, second_v] => {
            // Replace the first_v's value with the second one.
            *first_v = second_v.clone();
        },
    }
}

pub type ConfigGetResult<T> = Result<T, ConfigGetError>;

impl ConfigTable {
    fn _new(v: TomlMap<String, toml::Value>) -> Self {
        Self(v)
    }

    /// Merges ConfigTables
    pub fn merge(self, other: Self) -> Self {
        let mut inner_self_table = toml::Value::Table(self.0);
        let mut inner_other_table = toml::Value::Table(other.0);
        merge(&mut inner_self_table, &mut inner_other_table);

        if let toml::Value::Table(t) = inner_self_table {
            Self(t)
        } else {
            panic!("Should not occur")
        }
    }

    pub fn get<'de, T>(&self, key: &str) -> ConfigGetResult<T>
    where
        T: serde::Deserialize<'de>,
    {
        let mut current_cfg = &self.0;
        let mut tried_key_path = String::new();
        let mut key_parts = key.split('.').peekable();
        let res = loop {
            let kp = match key_parts.next() {
                None => break Err(ConfigGetError::KeyNotFound { keyname: key.to_string() }),
                Some(p) => p,
            };
            tried_key_path.push_str(kp);
            let current_value = match current_cfg.get(kp) {
                None => break Err(ConfigGetError::KeyNotFound { keyname: key.to_string() }),
                Some(v) => v,
            };
            if key_parts.peek().is_none() {
                // The next iteration is empty, we try to deserialise the current value
                break Deserialize::deserialize(current_value.clone()).map_err(|de_err| de_err.into());
            }
            // Case when there are more key parts, we try to extract the underlying toml table
            match current_value {
                toml::Value::Table(t) => {
                    current_cfg = t;
                    tried_key_path.push('.');
                    continue;
                },
                rest_val => {
                    break Err(ConfigGetError::KeyTableReference {
                        keyname: key.to_string(),
                        val: rest_val.to_string(),
                        tried_key_path,
                    });
                },
            }
        };
        if let Err(e) = &res {
            if !matches!(e, ConfigGetError::KeyNotFound { .. }) {
                error!(target:"server::loading", "bad value some kind of other error when retrieving config from this table: {e}");
            }
        }
        res
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Error reading from filesystem: {filepath:?}, err was: {err}")]
    Filesystem {
        filepath: PathBuf,
        #[source]
        err:      io::Error,
    },
    #[error("Error decoding from TOML: {filepath:?}, err was: {err}")]
    TOMLDecode {
        filepath: PathBuf,
        #[source]
        err:      toml::de::Error,
    },
    #[error("generic error: {msg}")]
    Generic { msg: String },
}

pub fn toml_from_filepath<C: serde::de::DeserializeOwned, P: AsRef<Path>>(filepath: P) -> Result<C, ConfigError> {
    let contents = match fs::read_to_string(filepath.as_ref()) {
        Err(e) => {
            return Err(ConfigError::Filesystem {
                filepath: filepath.as_ref().to_owned(),
                err:      e,
            })
        },
        Ok(s) => s,
    };
    match toml::from_str(contents.as_str()) {
        Err(e) => Err(ConfigError::TOMLDecode {
            filepath: filepath.as_ref().to_owned(),
            err:      e,
        }),
        Ok(c) => Ok(c),
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum LogLevel {
    Disabled = 0,
    Trace = 1,
    Debug = 2,
    Info = 3,
    Warning = 4,
    Error = 5,
    // Fatal = 6,
}

impl From<LogLevel> for Option<tracing::Level> {
    fn from(value: LogLevel) -> Self {
        use LogLevel::*;
        match value {
            Disabled => None,
            // Fatal => Some(tracing::Level::ERROR),
            Error => Some(tracing::Level::ERROR),
            Warning => Some(tracing::Level::WARN),
            Info => Some(tracing::Level::INFO),
            Debug => Some(tracing::Level::DEBUG),
            Trace => Some(tracing::Level::TRACE),
        }
    }
}

#[serde_inline_default]
#[derive(Deserialize, Serialize, DefaultFromSerde, Clone, PartialEq)]
#[expect(non_snake_case)]
pub struct DatabaseInfo {
    #[serde_inline_default("127.0.0.1:3306".to_string())]
    pub Address:      String,
    #[serde_inline_default("azcore".to_string())]
    pub User:         String,
    #[serde_inline_default("azcore".to_string())]
    pub Password:     String,
    #[serde_inline_default("".to_string())]
    pub DatabaseName: String,
}

impl Debug for DatabaseInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mysql://{}:<MASKED>@{}/{}", self.User, self.Address, self.DatabaseName)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum WrongPassBanType {
    BanIP,
    BanAccount,
}

#[serde_inline_default]
#[derive(Deserialize, Serialize, DefaultFromSerde, Clone, Debug, PartialEq)]
#[expect(non_snake_case)]
pub struct WrongPass {
    #[serde_inline_default(5)]
    pub MaxCount: u64,
    #[serde_inline_default(600)]
    pub BanTime:  u64,
    #[serde_inline_default(WrongPassBanType::BanIP)]
    pub BanType:  WrongPassBanType,
    #[serde_inline_default(false)]
    pub Logging:  bool,
}

impl DatabaseInfo {
    pub fn default_with_info(database: &str) -> Self {
        Self {
            DatabaseName: database.to_string(),
            ..Self::default()
        }
    }

    pub fn connect_url(&self) -> String {
        format!("mysql://{}:{}@{}/{}", self.User, self.Password, self.Address, self.DatabaseName)
    }

    pub fn connect_url_without_db(&self) -> String {
        format!("mysql://{}:{}@{}", self.User, self.Password, self.Address)
    }
}

flags! {
  #[derive(PartialOrd, Ord)]
  pub enum DatabaseType: u8 {
    #[allow(clippy::identity_op)]
    Login       = 0b0001,
    Character   = 0b0010,
    World       = 0b0100,
    Hotfix      = 0b1000,
    All = (DatabaseType::Login | DatabaseType::Character | DatabaseType::World | DatabaseType::Hotfix).bits(),
  }
}

impl DatabaseType {
    pub fn db_module_name(&self) -> Option<&'static str> {
        match *self {
            Self::All => None,
            Self::Character => Some("db-characters"),
            Self::Hotfix => Some("db-hotfixes"),
            Self::Login => Some("db-auth"),
            Self::World => Some("db-world"),
        }
    }

    pub fn base_files_directory(&self) -> Option<PathBuf> {
        self.db_module_name().map(|db_module_name| format!("data/sql/base/{db_module_name}").into())
    }
}

#[serde_inline_default]
#[derive(Deserialize, DefaultFromSerde, Serialize, Clone, Debug, PartialEq)]
pub struct DbUpdates {
    #[serde_inline_default(DatabaseType::All.into())]
    pub EnableDatabases:      FlagSet<DatabaseType>,
    #[serde_inline_default(true)]
    pub AutoSetup:            bool,
    #[serde_inline_default(true)]
    pub Redundancy:           bool,
    #[serde_inline_default(false)]
    pub ArchivedRedundancy:   bool,
    #[serde_inline_default(true)]
    pub AllowRehash:          bool,
    #[serde_inline_default(Some(3))]
    pub CleanDeadRefMaxCount: Option<isize>,
}

impl DbUpdates {
    pub fn update_enabled(&self, update_flags: impl Into<FlagSet<DatabaseType>>) -> bool {
        (self.EnableDatabases & update_flags).bits() > 0
    }

    pub fn should_cleanup(&self, applied_count: usize) -> bool {
        let Some(c) = &self.CleanDeadRefMaxCount else { return false };
        *c < 0 || applied_count <= usize::try_from(*c).unwrap()
    }
}

flags! {
    /// Flags: Define some extra modifications to do to logging message
    ///
    /// AddLogTimestamps (1) - Prefix Timestamp to the text
    ///
    /// AddLogLevel (2) - Prefix Log Level to the text
    ///
    /// AddLogFilter (4) - Prefix Log Filter type to the text
    ///
    /// AppendFileTimestamps (8) - Append timestamp to the log file name. This causes the file to roll daily
    ///
    /// TruncateFile (16) - Truncate file before writing
    ///
    /// BackupBeforeOverwrite (32) - Make a backup of existing file before overwrite, TruncateFile must be set
    pub enum LogFlags: u8 {
        AddLogTimestamps        = 0b000001,
        AddLogLevel             = 0b000010,
        AddLogFilter            = 0b000100,
        AppendFileTimestamps    = 0b001000,
        TruncateFile            = 0b010000,
        BackupBeforeOverwrite   = 0b100000,
    }
}

// #[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
// pub enum LogConsoleColours {
//     Black = 0,
//     Red = 1,
//     Green = 2,
//     Brown = 3,
//     Blue = 4,
//     Magenta = 5,
//     Cyan = 6,
//     Grey = 7,
//     Yellow = 8,
//     Lred = 9,
//     Lgreen = 10,
//     Lblue = 11,
//     Lmagenta = 12,
//     Lcyan = 13,
//     White = 14,
// }

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum LogAppender {
    Console {
        name:      String,
        min_level: LogLevel,
        max_level: LogLevel,
        flags:     FlagSet<LogFlags>,
        // colours: Vec<(LogLevel, LogConsoleColours)>,
    },
    File {
        name:      String,
        min_level: LogLevel,
        max_level: LogLevel,
        flags:     FlagSet<LogFlags>,
        file:      String,
        // mode:  String,
    },
    // TODO:
    // Db {},
}

pub fn default_worldserver_log_appenders() -> Vec<LogAppender> {
    // use LogConsoleColours::*;
    use LogFlags::*;
    use LogLevel::*;
    vec![
        LogAppender::Console {
            name:      String::from("Console"),
            min_level: Info,
            max_level: Error,
            flags:     AddLogLevel | AddLogFilter,
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
            name:      String::from("Server"),
            min_level: Warning,
            max_level: Error,
            flags:     AddLogLevel | AddLogFilter | AddLogTimestamps, // TruncateFile.into(),
            file:      String::from("Server.log"),
        },
        LogAppender::File {
            name:      String::from("GM"),
            min_level: Warning,
            max_level: Error,
            flags:     AddLogTimestamps | AddLogLevel | AddLogFilter | AppendFileTimestamps,
            file:      String::from("gm.log"),
        },
        LogAppender::File {
            name:      String::from("DBErrors"),
            min_level: Warning,
            max_level: Error,
            flags:     None.into(),
            file:      String::from("DBErrors.log"),
        },
    ]
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct LogLoggerConfig {
    pub name:      String,
    pub min_level: LogLevel,
    pub max_level: LogLevel,
    pub appenders: Vec<String>,
}

pub fn default_worldserver_log_configs() -> Vec<LogLoggerConfig> {
    use LogLevel::*;
    vec![
        LogLoggerConfig {
            name:      String::from("root"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("Server")],
        },
        LogLoggerConfig {
            name:      String::from("module"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("Server")],
        },
        LogLoggerConfig {
            name:      String::from("commands::gm"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("GM")],
        },
        LogLoggerConfig {
            name:      String::from("diff"),
            min_level: Warning,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("Server")],
        },
        LogLoggerConfig {
            name:      String::from("mmaps"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Server")],
        },
        LogLoggerConfig {
            name:      String::from("server"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("Server")],
        },
        LogLoggerConfig {
            name:      String::from("sql::sql"),
            min_level: Warning,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("DBErrors")],
        },
        LogLoggerConfig {
            name:      String::from("sql"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("Server")],
        },
        LogLoggerConfig {
            name:      String::from("time::update"),
            min_level: Info,
            max_level: Error,
            appenders: vec![String::from("Console"), String::from("Server")],
        },
    ]
}

#[cfg(test)]
mod tests {
    use flagset::FlagSet;

    use crate::configuration::*;

    fn configtable_helper(toml_str: &str) -> ConfigTable {
        ConfigTable::_new(toml::from_str(toml_str).unwrap())
    }

    #[test]
    fn configtable_deserialises_basic_types() {
        let t = configtable_helper(
            r#"
        a="string"
        b=2
        c=-3.155
        d=true
        # e=1979-05-26T15:32:00+08:00
        f=["s1", "s2", "s3"]
        [g]
        a=-1
        b="two"
        c=[3.1, 2.4, 4.5]
        "#,
        );
        assert_eq!(t.get::<String>("a").unwrap(), "string".to_string());
        assert_eq!(t.get::<u32>("b").unwrap(), 2);
        assert_eq!(t.get::<f64>("c").unwrap(), -3.155);
        assert!(t.get::<bool>("d").unwrap());
        // TODO: UNCOMMENT BELOW WHEN THIS IS FIXED: https://github.com/toml-rs/toml/issues/440
        // assert_eq!(
        //     t.get::<toml::value::Datetime>("e").unwrap(),
        //     toml::value::Datetime {
        //         date:   Some(toml::value::Date {
        //             year:  1979,
        //             month: 5,
        //             day:   26,
        //         }),
        //         time:   Some(toml::value::Time {
        //             hour:       15,
        //             minute:     32,
        //             second:     0,
        //             nanosecond: 0,
        //         }),
        //         offset: Some(toml::value::Offset::Custom { minutes: 8 }),
        //     }
        // );
        assert_eq!(t.get::<Vec<String>>("f").unwrap(), vec!["s1".to_string(), "s2".to_string(), "s3".to_string()]);
        assert_eq!(t.get::<i32>("g.a").unwrap(), -1);
        assert_eq!(t.get::<String>("g.b").unwrap(), "two".to_string());
        assert_eq!(t.get::<Vec<f32>>("g.c").unwrap(), vec![3.1, 2.4, 4.5]);
    }

    #[test]
    fn configtable_deserialises_serde_defined_types() {
        let t = configtable_helper(
            r#"
        [WorldDatabaseInfo]
        Address="127.0.0.1:3306"
        User="azcore"
        Password="azcore"
        DatabaseName="azcore_world"

        [[Appender]]
        type = "Console"
        name = "Console"
        min_level = "Info"
        max_level = "Error"
        flags = 6
        
        [[Appender]]
        type = "File"
        name = "Server"
        min_level = "Warning"
        max_level = "Error"
        flags = 7
        file = "Server.log"
        
        [[Appender]]
        type = "File"
        name = "GM"
        min_level = "Warning"
        max_level = "Error"
        flags = 15
        file = "gm.log"
        
        [[Appender]]
        type = "File"
        name = "DBErrors"
        min_level = "Warning"
        max_level = "Error"
        flags = 0
        file = "DBErrors.log"

        [[Logger]]
        name="root"
        min_level="Info"
        max_level="Error"
        appenders=["Console", "Server"]

        [[Logger]]
        name="module"
        min_level="Info"
        max_level="Error"
        appenders=["Console", "Server"]

        [[Logger]]
        name="commands::gm"
        min_level="Info"
        max_level="Error"
        appenders=["Console", "GM"]

        [[Logger]]
        name="diff"
        min_level="Warning"
        max_level="Error"
        appenders=["Console", "Server"]

        [[Logger]]
        name="mmaps"
        min_level="Info"
        max_level="Error"
        appenders=["Server"]

        [[Logger]]
        name="server"
        min_level="Info"
        max_level="Error"
        appenders=["Console", "Server"]

        [[Logger]]
        name="sql::sql"
        min_level="Warning"
        max_level="Error"
        appenders=["Console", "DBErrors"]

        [[Logger]]
        name="sql"
        min_level="Info"
        max_level="Error"
        appenders=["Console", "Server"]

        [[Logger]]
        name="time::update"
        min_level="Info"
        max_level="Error"
        appenders=["Console", "Server"]
        
        [Updates]
        EnableDatabases = 7
        AutoSetup = true
        Redundancy = true
        ArchivedRedundancy = false
        AllowRehash = true
        CleanDeadRefMaxCount = 3
        "#,
        );

        // assert_eq!(
        //     t.get::<DatabaseInfo>("WorldDatabaseInfo").unwrap(),
        //     DatabaseInfo::default_with_info("azcore_world", "data/sql/base/db_world", "db-world")
        // );
        // assert_eq!(t.get::<Vec<LogAppender>>("Appender").unwrap(), default_worldserver_log_appenders());
        // assert_eq!(t.get::<Vec<LogLoggerConfig>>("Logger").unwrap(), default_worldserver_log_configs());
        assert_eq!(
            t.get::<DbUpdates>("Updates").unwrap(),
            DbUpdates {
                EnableDatabases:      (DatabaseType::Login | DatabaseType::Character | DatabaseType::World),
                AutoSetup:            true,
                Redundancy:           true,
                ArchivedRedundancy:   false,
                AllowRehash:          true,
                CleanDeadRefMaxCount: Some(3),
            }
        )
    }

    #[test]
    fn configtable_expects_errors() {
        #[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
        struct A {
            a: String,
        }

        let t = configtable_helper(
            r#"
            nested_not_found.key1 = "key1"
            serde_error="string"
            serde_error2.table = { a = "b" }
            nested_bad_table_reference.key1 = "key1key2"
            "#,
        );
        assert!(matches!(t.get::<i32>("not_found"), Err(ConfigGetError::KeyNotFound { keyname }) if keyname == *"not_found"));
        assert!(matches!(t.get::<i32>("nested_not_found.key2"), Err(ConfigGetError::KeyNotFound { keyname }) if keyname == *"nested_not_found.key2"));
        assert!(matches!(t.get::<i32>("serde_error"), Err(ConfigGetError::TomlDeserialisation(_))));
        assert!(matches!(t.get::<String>("serde_error2"), Err(ConfigGetError::TomlDeserialisation(_))));
        assert!(matches!(t.get::<A>("serde_error2.table"), Ok(v) if v == A { a: "b".to_string() }));
        assert!(matches!(t.get::<A>("serde_error3"), Err(ConfigGetError::KeyNotFound { keyname }) if keyname == *"serde_error3"));
        assert!(matches!(
            t.get::<String>("nested_bad_table_reference.key1.key2"),
            Err(
                ConfigGetError::KeyTableReference { keyname, tried_key_path, val })
                    if keyname == *"nested_bad_table_reference.key1.key2" && val == *"\"key1key2\"" && tried_key_path == *"nested_bad_table_reference.key1",
        ));
    }

    #[test]
    fn it_sanity_checks_database_type_flags() {
        use DatabaseType::*;
        assert_eq!(FlagSet::from(Login).bits(), 1);
        assert_eq!(FlagSet::from(Character).bits(), 2);
        assert_eq!(FlagSet::from(World).bits(), 4);
        assert_eq!(FlagSet::from(Hotfix).bits(), 8);
        assert_eq!(FlagSet::from(All).bits(), 15);
    }

    #[test]
    fn it_checks_updates_enabled() {
        use DatabaseType::*;
        let mut u = DbUpdates {
            EnableDatabases: FlagSet::from(None),
            ..Default::default()
        };
        assert!(!u.update_enabled(Login));
        assert!(!u.update_enabled(Character));
        assert!(!u.update_enabled(World));
        assert!(!u.update_enabled(All));
        u.EnableDatabases = FlagSet::from(Login);
        assert!(u.update_enabled(Login));
        assert!(!u.update_enabled(Character));
        assert!(!u.update_enabled(World));
        assert!(u.update_enabled(All));
        u.EnableDatabases = FlagSet::from(Character);
        assert!(!u.update_enabled(Login));
        assert!(u.update_enabled(Character));
        assert!(!u.update_enabled(World));
        assert!(u.update_enabled(All));
        u.EnableDatabases = FlagSet::from(World);
        assert!(!u.update_enabled(Login));
        assert!(!u.update_enabled(Character));
        assert!(u.update_enabled(World));
        assert!(u.update_enabled(All));
        u.EnableDatabases = FlagSet::from(All);
        assert!(u.update_enabled(Login));
        assert!(u.update_enabled(Character));
        assert!(u.update_enabled(World));
        assert!(u.update_enabled(All));
    }
}
