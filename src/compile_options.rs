use const_format::formatcp;
use git_version::git_version;

pub const GIT_VERSION: &str = git_version!(
    args = ["--always", "--dirty=-modified", "--abbrev=10"],
    prefix = "git:",
    cargo_prefix = "cargo:",
    fallback = "unknown"
);
pub const GIT_HASH: &str = git_version!(args = ["--abbrev=10", "--always"]);

#[cfg(target_os = "windows")]
pub const CONF_DIR: &str = option_env!("CONF_DIR").unwrap_or("configs\\");

#[cfg(target_os = "linux")]
pub const CONF_DIR: &str = option_env!("CONF_DIR").unwrap_or("env/dist/etc");
#[cfg(target_os = "windows")]
pub const CONF_MODULES_DIR: &str = option_env!("CONF_MODULES_DIR").unwrap_or(formatcp!("{CONF_DIR}\\modules"));
#[cfg(target_os = "linux")]
pub const CONF_MODULES_DIR: &str = option_env!("CONF_MODULES_DIR").unwrap_or(formatcp!("{CONF_DIR}/modules"));

pub const AZOTHA_CORE_CONFIG: &str = option_env!("AZOTHA_CORE_CONFIG").unwrap_or("app-worldserver.toml");
pub const AZOTHA_REALM_CONFIG: &str = option_env!("AZOTHA_REALM_CONFIG").unwrap_or("app-authserver.toml");
pub const AZOTHA_DB_IMPORT_CONFIG: &str = option_env!("AZOTHA_DB_IMPORT_CONFIG").unwrap_or("tools-dbimport.toml");
