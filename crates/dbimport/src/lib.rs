#![feature(lint_reasons)]

use std::path::PathBuf;

use azothacore_common::{
    configuration::{Config, DatabaseInfo, DbUpdates, LogAppender, LogFlags, LogLevel, LogLoggerConfig},
    log::LoggingConfig,
};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

structstruck::strike! {
    #[strikethrough[serde_inline_default]]
    #[strikethrough[expect(non_snake_case)]]
    #[strikethrough[derive(DefaultFromSerde, Deserialize, Serialize, Clone, Debug, PartialEq)]]
    pub struct DbImportConfig {
        /// Logs directory path - all logs will be written inside this directory.
        #[serde_inline_default("logs".into())] pub LogsDir: PathBuf,
        #[serde(default="default_dbimport_log_appenders")] pub Appender: Vec<LogAppender>,
        #[serde(default="default_dbimport_log_configs")] pub Logger: Vec<LogLoggerConfig>,
        /// Database connection settings for the realm server.
        #[serde_inline_default(DatabaseInfo::default_with_info("azcore_auth"))] pub LoginDatabaseInfo: DatabaseInfo,
        /// Database connection settings for the world.
        #[serde_inline_default(DatabaseInfo::default_with_info("azcore_world"))] pub WorldDatabaseInfo: DatabaseInfo,
        /// Database connection settings for the characters and accounts.
        #[serde_inline_default(DatabaseInfo::default_with_info("azcore_characters"))] pub CharacterDatabaseInfo: DatabaseInfo,
        /// Database connection settings containing hotfixes to existing client DB2 data.
        #[serde_inline_default(DatabaseInfo::default_with_info("azcore_hotfixes"))] pub HotfixDatabaseInfo: DatabaseInfo,
        /// Database Update settings
        #[serde(default)] pub Updates: DbUpdates,
    }
}

impl Config for DbImportConfig {}

pub fn default_dbimport_log_appenders() -> Vec<LogAppender> {
    // use LogConsoleColours::*;
    use LogFlags::*;
    use LogLevel::*;
    vec![
        LogAppender::Console {
            name:      String::from("Console"),
            min_level: Info,
            max_level: Error,
            flags:     AddLogLevel | AddLogFilter | TruncateFile | BackupBeforeOverwrite,
        },
        LogAppender::File {
            name:      String::from("DBImport"),
            min_level: Warning,
            max_level: Error,
            flags:     AddLogLevel | AddLogFilter | TruncateFile | BackupBeforeOverwrite | AddLogTimestamps,
            file:      String::from("DBImport.log"),
        },
    ]
}

pub fn default_dbimport_log_configs() -> Vec<LogLoggerConfig> {
    use LogLevel::*;
    vec![LogLoggerConfig {
        name:      String::from("root"),
        min_level: Debug,
        max_level: Error,
        appenders: vec![String::from("Console"), String::from("DBImport")],
    }]
}

impl LoggingConfig for DbImportConfig {
    fn retrieve_appenders(&self) -> &[azothacore_common::configuration::LogAppender] {
        &self.Appender
    }

    fn retrieve_loggers(&self) -> &[azothacore_common::configuration::LogLoggerConfig] {
        &self.Logger
    }

    fn retrieve_logs_dir(&self) -> PathBuf {
        self.LogsDir.clone()
    }
}
