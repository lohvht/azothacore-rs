use const_format::formatcp;
use git_version::git_version;

pub const GIT_VERSION: &str = git_version!(
    args = ["--always", "--dirty=-modified", "--abbrev=10"],
    prefix = "git:",
    cargo_prefix = "cargo:",
    fallback = "unknown"
);
pub const GIT_HASH: &str = git_version!(args = ["--abbrev=10", "--always"]);

const fn unwrap_default(o: Option<&'static str>, s: &'static str) -> &'static str {
    match o {
        None => s,
        Some(s) => s,
    }
}

#[cfg(target_os = "windows")]
pub const CONF_DIR: &str = unwrap_default(option_env!("CONF_DIR"), "configs\\");

#[cfg(target_os = "linux")]
pub const CONF_DIR: &str = unwrap_default(option_env!("CONF_DIR"), "env/etc");
#[cfg(target_os = "windows")]
pub const CONF_MODULES_DIR: &str = unwrap_default(option_env!("CONF_MODULES_DIR"), formatcp!("{CONF_DIR}\\modules"));
#[cfg(target_os = "linux")]
pub const CONF_MODULES_DIR: &str = unwrap_default(option_env!("CONF_MODULES_DIR"), formatcp!("{CONF_DIR}/modules"));

pub const AZOTHA_CORE_CONFIG: &str = unwrap_default(option_env!("AZOTHA_CORE_CONFIG"), "app-worldserver.toml");
pub const AZOTHA_REALM_CONFIG: &str = unwrap_default(option_env!("AZOTHA_REALM_CONFIG"), "app-authserver.toml");
pub const AZOTHA_DB_IMPORT_CONFIG: &str = unwrap_default(option_env!("AZOTHA_DB_IMPORT_CONFIG"), "tools-dbimport.toml");
pub const AZOTHA_FULL_EXTRACTOR_CONFIG: &str = unwrap_default(option_env!("AZOTHA_FULL_EXTRACTOR_CONFIG"), "extractor.toml");
