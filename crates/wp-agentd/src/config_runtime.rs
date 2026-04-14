//! Runtime config loading and mode selection.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use wp_agent_contracts::agent_config::AgentConfigContract;
use wp_agent_shared::fs::write_bytes_atomic;
use wp_agent_shared::paths::AGENT_CONFIG_FILE;
use wp_agent_validate::config::validate_config;

#[path = "config_runtime_support.rs"]
mod support;

use support::{default_file_config_text, expand_env_contract, resolve_paths};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnsuredConfigFile {
    pub path: PathBuf,
    pub created: bool,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    ParseToml(toml::de::Error),
    MissingEnvVar(String),
    Validation(&'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::ParseToml(err) => write!(f, "config parse error: {err}"),
            Self::MissingEnvVar(name) => write!(f, "missing environment variable: {name}"),
            Self::Validation(code) => write!(f, "config validation failed: {code}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn load_or_init(config_root: &Path) -> Result<AgentConfigContract, ConfigError> {
    let ensured = ensure_default_config(config_root)?;
    load_from_path(&ensured.path)
}

pub fn ensure_default_config(config_root: &Path) -> Result<EnsuredConfigFile, ConfigError> {
    fs::create_dir_all(config_root)?;
    let config_path = config_root.join(AGENT_CONFIG_FILE);
    let created = if config_path.exists() {
        false
    } else {
        let text = default_file_config_text();
        write_bytes_atomic(&config_path, text.as_bytes())?;
        true
    };
    Ok(EnsuredConfigFile {
        path: config_path,
        created,
    })
}

pub fn default_config_template() -> String {
    default_file_config_text()
}

pub fn load_from_path(config_path: &Path) -> Result<AgentConfigContract, ConfigError> {
    let text = fs::read_to_string(config_path)?;
    let parsed = toml::from_str::<AgentConfigContract>(&text).map_err(ConfigError::ParseToml)?;
    let env_resolved = expand_env_contract(parsed)?;
    let path_resolved = resolve_paths(env_resolved, config_path);
    validate_config(&path_resolved).map_err(|err| ConfigError::Validation(err.code))?;
    Ok(path_resolved)
}

#[cfg(test)]
#[path = "config_runtime_tests.rs"]
mod tests;
