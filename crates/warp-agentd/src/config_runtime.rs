//! Runtime config loading and mode selection.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use warp_insight_contracts::agent_config::{AgentConfigContract, LogFileInputsFile};
use warp_insight_shared::fs::write_bytes_atomic;
use warp_insight_shared::paths::{AGENTD_CONFIG_FILE, LEGACY_AGENT_CONFIG_FILE};
use warp_insight_validate::config::validate_config;

#[path = "config_runtime_support.rs"]
mod support;

use support::{
    absolutize, default_file_config_text, expand_env_contract, expand_string, resolve_paths,
};

#[derive(Debug, Clone, PartialEq, Eq, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Discovery", module = "Discovery.Config")]
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
    let config_path = resolve_config_path(config_root);
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
    let mut parsed =
        toml::from_str::<AgentConfigContract>(&text).map_err(ConfigError::ParseToml)?;
    load_file_inputs_from_task_file(&mut parsed, config_path)?;
    let env_resolved = expand_env_contract(parsed)?;
    let path_resolved = resolve_paths(env_resolved, config_path);
    validate_config(&path_resolved).map_err(|err| ConfigError::Validation(err.code))?;
    Ok(path_resolved)
}

/// 若声明了 `[telemetry.logs] file_inputs_file`，从该外置任务清单加载 `file_inputs`。
///
/// 任务清单路径相对本配置文件解析（支持 `${ENV}` 展开），与内联
/// `[[telemetry.logs.file_inputs]]` 互斥：同时出现时拒绝加载，避免来源不明。
fn load_file_inputs_from_task_file(
    config: &mut AgentConfigContract,
    config_path: &Path,
) -> Result<(), ConfigError> {
    let Some(raw_path) = config.telemetry.logs.file_inputs_file.take() else {
        return Ok(());
    };
    let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    let task_path = absolutize(config_dir, &expand_string(raw_path)?);
    let text = fs::read_to_string(&task_path).map_err(|err| {
        ConfigError::Io(io::Error::new(
            err.kind(),
            format!("read file_inputs_file {}: {err}", task_path.display()),
        ))
    })?;
    if !config.telemetry.logs.file_inputs.is_empty() {
        return Err(ConfigError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "file_inputs_file {} conflicts with inline [[telemetry.logs.file_inputs]]; declare one or the other",
                task_path.display()
            ),
        )));
    }
    let tasks = toml::from_str::<LogFileInputsFile>(&text).map_err(|err| {
        ConfigError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("parse file_inputs_file {}: {err}", task_path.display()),
        ))
    })?;
    config.telemetry.logs.file_inputs = tasks.file_inputs;
    Ok(())
}

pub fn resolve_config_path(config_root: &Path) -> PathBuf {
    let preferred = config_root.join(AGENTD_CONFIG_FILE);
    if preferred.is_file() {
        return preferred;
    }

    let legacy = config_root.join(LEGACY_AGENT_CONFIG_FILE);
    if legacy.is_file() {
        return legacy;
    }

    preferred
}

#[cfg(test)]
#[path = "config_runtime_tests.rs"]
mod tests;
