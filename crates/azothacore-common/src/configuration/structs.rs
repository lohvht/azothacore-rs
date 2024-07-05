use std::{
    fmt::Debug,
    hash::Hash,
    path::{Path, PathBuf},
};

use figment::{
    providers::{Env, Format, Toml},
    Figment,
    Profile,
};
use flagset::{flags, FlagSet};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

use crate::{AzResult, BASE_DIR};

/// Gets a given config from both the env var and a toml file
/// For env vars, key paths are split by double underscores "__"
/// and are CASE-SENSITIVE
pub fn from_env_toml<C: serde::de::DeserializeOwned, P: AsRef<Path>>(filepath: P) -> AzResult<C> {
    let fig = Figment::new()
        .merge(Toml::file(filepath))
        .admerge(Env::prefixed("AZ__").split("__").lowercase(false));

    Ok(fig.extract().map_err(|mut e| {
        // Replace the figment profile here b/c the error message produced
        //
        // and thus default to the default config, but the error message printed will
        // contain the "default" prefix if deserialised via toml.
        e.profile = Some(Profile::new(""));
        e
    })?)
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
    #[serde(default)]
    pub DatabaseName: String,
}

impl Debug for DatabaseInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mysql://{}:<MASKED>@{}/{}", self.User, self.Address, self.DatabaseName)
    }
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
  }
}

impl DatabaseType {
    pub fn db_module_name(&self) -> Option<&'static str> {
        match *self {
            Self::Character => Some("db-characters"),
            Self::Hotfix => Some("db-hotfixes"),
            Self::Login => Some("db-auth"),
            Self::World => Some("db-world"),
        }
    }

    pub fn base_files_directory(&self) -> Option<PathBuf> {
        self.db_module_name()
            .map(|db_module_name| format!("{BASE_DIR}/data/sql/base/{db_module_name}").into())
    }
}

#[serde_inline_default]
#[derive(Deserialize, DefaultFromSerde, Serialize, Clone, Debug, PartialEq)]
pub struct DbUpdates {
    /// A mask that describes which databases shall be updated
    #[serde_inline_default(FlagSet::full())]
    pub EnableDatabases:      FlagSet<DatabaseType>,
    /// Auto populate empty databases.
    #[serde_inline_default(true)]
    pub AutoSetup:            bool,
    /// Perform data redundancy checks through hashing
    ///
    /// to detect changes on sql updates and reapply it.
    #[serde_inline_default(true)]
    pub Redundancy:           bool,
    /// Check hashes of archived updates (slows down startup).
    #[serde_inline_default(false)]
    pub ArchivedRedundancy:   bool,
    /// Inserts the current file hash in the database if it is left empty.
    ///
    /// Useful if you want to mark a file as applied but you don't know its hash.
    #[serde_inline_default(true)]
    pub AllowRehash:          bool,
    /// Cleans dead/ orphaned references that occur if an update was removed or renamed and edited in one step.
    ///
    /// It only starts the clean up if the count of the missing updates is below or equal the Updates.CleanDeadRefMaxCount value.
    ///
    /// This way prevents erasing of the update history due to wrong source directory state (maybe wrong branch or bad revision).
    ///
    /// Disable this if you want to know if the database is in a possible "dirty state".
    #[serde_inline_default(3)]
    pub CleanDeadRefMaxCount: isize,
}

impl DbUpdates {
    pub fn update_enabled(&self, update_flags: impl Into<FlagSet<DatabaseType>>) -> bool {
        (self.EnableDatabases & update_flags).bits() > 0
    }

    pub fn should_cleanup(&self, applied_count: usize) -> bool {
        if self.CleanDeadRefMaxCount == 0 {
            return false;
        }
        self.CleanDeadRefMaxCount < 0 || applied_count <= usize::try_from(self.CleanDeadRefMaxCount).unwrap()
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct LogLoggerConfig {
    pub name:      String,
    pub min_level: LogLevel,
    pub max_level: LogLevel,
    pub appenders: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use figment::Jail;
    use flagset::FlagSet;
    use serde::{Deserialize, Serialize};
    use serde_default::DefaultFromSerde;
    use serde_inline_default::serde_inline_default;

    use crate::{bounded_nums::RangedBoundedNum, configuration::*, durationb_mins, durationb_s, f32b};

    #[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
    enum TestEnum {
        Variant0,
        Variant1,
        Variant2,
    }

    #[serde_inline_default]
    #[derive(DefaultFromSerde, Deserialize, Serialize, Clone, Debug, PartialEq)]
    struct ChildConfigTest {
        #[serde_inline_default("some_path".into())]
        Path:          PathBuf,
        #[serde(default)]
        OptionaUInt32: Option<u32>,
    }

    #[serde_inline_default]
    #[derive(DefaultFromSerde, Deserialize, Serialize, Clone, Debug, PartialEq)]
    struct ChildConfigTestWithNestedStruct {
        #[serde(default)]
        Duration:        RangedBoundedNum<Duration, { durationb_s!(0) }, { durationb_mins!(15) }, { durationb_mins!(15) }>,
        #[serde(default)]
        ChildConfigTest: ChildConfigTest,
    }

    structstruck::strike! {
    #[strikethrough[serde_inline_default]]
    #[strikethrough[derive(DefaultFromSerde, Deserialize, Serialize, Clone, Debug, PartialEq)]]
    struct ConfigTest {
        #[serde_inline_default("MyString".into())] Str: String,
        #[serde_inline_default(44)] UInt32: u32,
        #[serde_inline_default(9.1)] F32: f32,
        #[serde_inline_default(true)] Bool: bool,
        #[serde(default)] BoundedUint32: RangedBoundedNum<u32, 10, 150, 75>,
        #[serde(default)] BoundedF32:  RangedBoundedNum<f32, { f32b!(0.0) }, { f32b!(100.0) }, { f32b!(95.0) }>,
        #[serde_inline_default(TestEnum::Variant2)] Enum: TestEnum,
        #[serde_inline_default([1, 2, 3, 4])] Array: [u32; 4],
        #[serde(default)] Nested: ChildConfigTestWithNestedStruct,
        #[serde_inline_default(vec![
            ChildConfigTest{
                Path: "One".into(),
                OptionaUInt32: Some(11),
            },
            ChildConfigTest{
                Path: "Two".into(),
                OptionaUInt32: Some(22),
            },
        ])] ChildConfigs: Vec<ChildConfigTest>,
    }
    }

    #[test]
    fn from_env_toml_deserialises_default() {
        let cfg: ConfigTest = from_env_toml("config.toml").unwrap();

        let default_cfg = ConfigTest::default();

        assert_eq!(cfg, default_cfg);

        let expected_cfg = ConfigTest {
            Str:           "MyString".into(),
            UInt32:        44,
            F32:           9.1,
            Bool:          true,
            BoundedUint32: 75.into(),
            BoundedF32:    95.0.into(),
            Enum:          TestEnum::Variant2,
            Array:         [1, 2, 3, 4],
            Nested:        ChildConfigTestWithNestedStruct {
                Duration:        Duration::from_secs(15 * 60).into(),
                ChildConfigTest: ChildConfigTest {
                    Path:          "some_path".into(),
                    OptionaUInt32: None,
                },
            },
            ChildConfigs:  vec![
                ChildConfigTest {
                    Path:          "One".into(),
                    OptionaUInt32: Some(11),
                },
                ChildConfigTest {
                    Path:          "Two".into(),
                    OptionaUInt32: Some(22),
                },
            ],
        };
        assert_eq!(cfg, expected_cfg);
    }

    #[test]
    fn from_env_toml_deserialises_with_toml_overrides() {
        Jail::expect_with(|jail| {
            jail.create_file(
                "config.toml",
                r#"
                Str = "TOML_OVERWRITTEN"
                [[ChildConfigs]]
                Path = "path_overwritten"
                OptionaUInt32 = 1

                [Nested]
                Duration = "5s"

                [Nested.ChildConfigTest]
                OptionaUInt32 = 7
            "#,
            )?;

            let cfg: ConfigTest = from_env_toml("config.toml").unwrap();

            let expected_cfg = ConfigTest {
                Str: "TOML_OVERWRITTEN".into(),
                //
                ChildConfigs: vec![ChildConfigTest {
                    Path:          "path_overwritten".into(),
                    OptionaUInt32: Some(1),
                }],
                Nested: ChildConfigTestWithNestedStruct {
                    Duration:        Duration::from_secs(5).into(),
                    ChildConfigTest: ChildConfigTest {
                        OptionaUInt32: Some(7),
                        ..Default::default()
                    },
                },
                ..Default::default()
            };

            assert_eq!(cfg, expected_cfg);

            Ok(())
        });
    }

    #[test]
    fn from_env_toml_deserialises_with_env_overrides() {
        Jail::expect_with(|jail| {
            jail.set_env("AZ__Str", "ENV_OVERWRITTEN");
            jail.set_env("AZ__Enum", "Variant1");
            jail.set_env("AZ__ChildConfigs", r#"[{Path="path_appended",OptionaUInt32=1}]"#);
            jail.set_env("AZ__Nested__Duration", "5s");
            jail.set_env("AZ__Nested__ChildConfigTest__OptionaUInt32", "7");

            let cfg: ConfigTest = from_env_toml("config.toml").unwrap();

            let expected_cfg = ConfigTest {
                Str: "ENV_OVERWRITTEN".into(),
                Enum: TestEnum::Variant1,
                ChildConfigs: vec![ChildConfigTest {
                    Path:          "path_appended".into(),
                    OptionaUInt32: Some(1),
                }],
                Nested: ChildConfigTestWithNestedStruct {
                    Duration:        Duration::from_secs(5).into(),
                    ChildConfigTest: ChildConfigTest {
                        OptionaUInt32: Some(7),
                        ..Default::default()
                    },
                },
                ..Default::default()
            };

            assert_eq!(cfg, expected_cfg);

            Ok(())
        });
    }

    #[test]
    fn from_env_toml_deserialises_with_both_toml_and_env_overrides() {
        Jail::expect_with(|jail| {
            jail.create_file(
                "config.toml",
                r#"
                Str = "TOML_OVERWRITTEN"
                BoundedF32 = 56
                [[ChildConfigs]]
                Path = "path_overwritten"
                OptionaUInt32 = 1
            "#,
            )?;

            jail.set_env("AZ__Str", "ENV_OVERWRITTEN");
            jail.set_env("AZ__Enum", "Variant1");
            jail.set_env("AZ__ChildConfigs", r#"[{Path="path_appended",OptionaUInt32=1}]"#);
            jail.set_env("AZ__Nested__Duration", "5s");
            jail.set_env("AZ__Nested__ChildConfigTest__OptionaUInt32", "7");

            let cfg: ConfigTest = from_env_toml("config.toml").unwrap();
            let expected_cfg = ConfigTest {
                Str: "ENV_OVERWRITTEN".into(),
                Enum: TestEnum::Variant1,
                BoundedF32: 56.0.into(),
                ChildConfigs: vec![
                    ChildConfigTest {
                        Path:          "path_overwritten".into(),
                        OptionaUInt32: Some(1),
                    },
                    ChildConfigTest {
                        Path:          "path_appended".into(),
                        OptionaUInt32: Some(1),
                    },
                ],
                Nested: ChildConfigTestWithNestedStruct {
                    Duration:        Duration::from_secs(5).into(),
                    ChildConfigTest: ChildConfigTest {
                        OptionaUInt32: Some(7),
                        ..Default::default()
                    },
                },
                ..Default::default()
            };

            assert_eq!(cfg, expected_cfg);

            Ok(())
        });
    }

    #[test]
    fn it_sanity_checks_database_type_flags() {
        use DatabaseType::*;
        assert_eq!(FlagSet::from(Login).bits(), 1);
        assert_eq!(FlagSet::from(Character).bits(), 2);
        assert_eq!(FlagSet::from(World).bits(), 4);
        assert_eq!(FlagSet::from(Hotfix).bits(), 8);
        assert_eq!(FlagSet::<DatabaseType>::full().bits(), 15);
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
        assert!(!u.update_enabled(FlagSet::full()));
        u.EnableDatabases = FlagSet::from(Login);
        assert!(u.update_enabled(Login));
        assert!(!u.update_enabled(Character));
        assert!(!u.update_enabled(World));
        assert!(u.update_enabled(FlagSet::full()));
        u.EnableDatabases = FlagSet::from(Character);
        assert!(!u.update_enabled(Login));
        assert!(u.update_enabled(Character));
        assert!(!u.update_enabled(World));
        assert!(u.update_enabled(FlagSet::full()));
        u.EnableDatabases = FlagSet::from(World);
        assert!(!u.update_enabled(Login));
        assert!(!u.update_enabled(Character));
        assert!(u.update_enabled(World));
        assert!(u.update_enabled(FlagSet::full()));
        u.EnableDatabases = FlagSet::<DatabaseType>::full();
        assert!(u.update_enabled(Login));
        assert!(u.update_enabled(Character));
        assert!(u.update_enabled(World));
        assert!(u.update_enabled(FlagSet::full()));
    }
}
