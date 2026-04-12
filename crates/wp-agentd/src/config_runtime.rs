//! Runtime config loading and mode selection.

use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use wp_agent_contracts::agent_config::{
    AgentConfigContract, AgentSection, ControlPlaneSection, ExecutionSection, PathsSection,
};
use wp_agent_shared::fs::write_bytes_atomic;
use wp_agent_shared::paths::{AGENT_CONFIG_FILE, LOG_DIR, RUN_DIR, STATE_DIR};
use wp_agent_validate::config::validate_config;

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    ParseToml(toml::de::Error),
    SerializeToml(toml::ser::Error),
    MissingEnvVar(String),
    Validation(&'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::ParseToml(err) => write!(f, "config parse error: {err}"),
            Self::SerializeToml(err) => write!(f, "config serialization error: {err}"),
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
    fs::create_dir_all(config_root)?;
    let config_path = config_root.join(AGENT_CONFIG_FILE);
    if !config_path.exists() {
        let default = default_file_config();
        let text = toml::to_string_pretty(&default).map_err(ConfigError::SerializeToml)?;
        write_bytes_atomic(&config_path, text.as_bytes())?;
    }
    load_from_path(&config_path)
}

pub fn load_from_path(config_path: &Path) -> Result<AgentConfigContract, ConfigError> {
    let text = fs::read_to_string(config_path)?;
    let parsed = toml::from_str::<AgentConfigContract>(&text).map_err(ConfigError::ParseToml)?;
    let env_resolved = expand_env_contract(parsed)?;
    let path_resolved = resolve_paths(env_resolved, config_path);
    validate_config(&path_resolved).map_err(|err| ConfigError::Validation(err.code))?;
    Ok(path_resolved)
}

fn default_file_config() -> AgentConfigContract {
    AgentConfigContract::new(
        AgentSection {
            agent_id: None,
            environment_id: None,
            instance_name: Some("local".to_string()),
        },
        ControlPlaneSection {
            enabled: false,
            endpoint: None,
            tls_mode: None,
            auth_mode: None,
        },
        PathsSection {
            root_dir: ".".to_string(),
            run_dir: RUN_DIR.to_string(),
            state_dir: STATE_DIR.to_string(),
            log_dir: LOG_DIR.to_string(),
        },
        ExecutionSection {
            max_running_actions: 1,
            cancel_grace_ms: 5_000,
            default_stdout_limit_bytes: 1_048_576,
            default_stderr_limit_bytes: 1_048_576,
        },
    )
}

fn expand_env_contract(
    mut config: AgentConfigContract,
) -> Result<AgentConfigContract, ConfigError> {
    config.agent.agent_id = expand_optional(config.agent.agent_id)?;
    config.agent.environment_id = expand_optional(config.agent.environment_id)?;
    config.agent.instance_name = expand_optional(config.agent.instance_name)?;
    config.control_plane.endpoint = expand_optional(config.control_plane.endpoint)?;
    config.control_plane.tls_mode = expand_optional(config.control_plane.tls_mode)?;
    config.control_plane.auth_mode = expand_optional(config.control_plane.auth_mode)?;
    config.paths.root_dir = expand_string(config.paths.root_dir)?;
    config.paths.run_dir = expand_string(config.paths.run_dir)?;
    config.paths.state_dir = expand_string(config.paths.state_dir)?;
    config.paths.log_dir = expand_string(config.paths.log_dir)?;
    Ok(config)
}

fn resolve_paths(mut config: AgentConfigContract, config_path: &Path) -> AgentConfigContract {
    let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    let root_dir = absolutize(config_dir, &config.paths.root_dir);

    config.paths.root_dir = root_dir.display().to_string();
    config.paths.run_dir = absolutize(&root_dir, &config.paths.run_dir)
        .display()
        .to_string();
    config.paths.state_dir = absolutize(&root_dir, &config.paths.state_dir)
        .display()
        .to_string();
    config.paths.log_dir = absolutize(&root_dir, &config.paths.log_dir)
        .display()
        .to_string();
    config
}

fn absolutize(base: &Path, raw: &str) -> PathBuf {
    let path = Path::new(raw);
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    normalize_path(joined)
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn expand_optional(value: Option<String>) -> Result<Option<String>, ConfigError> {
    value.map(expand_string).transpose()
}

fn expand_string(value: String) -> Result<String, ConfigError> {
    let mut out = String::with_capacity(value.len());
    let mut cursor = 0usize;
    while let Some(start) = value[cursor..].find("${") {
        let start = cursor + start;
        out.push_str(&value[cursor..start]);
        let rest = &value[start + 2..];
        let Some(end_rel) = rest.find('}') else {
            out.push_str(&value[start..]);
            return Ok(out);
        };
        let end = start + 2 + end_rel;
        let name = &value[start + 2..end];
        let expanded = env::var(name).map_err(|_| ConfigError::MissingEnvVar(name.to_string()))?;
        out.push_str(&expanded);
        cursor = end + 1;
    }
    out.push_str(&value[cursor..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{load_from_path, load_or_init};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("wp-agentd-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn load_or_init_creates_default_config() {
        let root = temp_dir("init");
        let config = load_or_init(&root).expect("load or init");

        assert!(root.join("agent.toml").exists());
        assert_eq!(config.paths.root_dir, root.display().to_string());
        assert_eq!(config.paths.run_dir, root.join("run").display().to_string());
    }

    #[test]
    fn load_from_path_expands_env_and_resolves_paths() {
        let root = temp_dir("load");
        let config_path = root.join("agent.toml");
        let home = std::env::var("HOME").expect("HOME");
        fs::write(
            &config_path,
            format!(
                r#"
schema_version = "v1alpha1"

[agent]
environment_id = "prod"
instance_name = "${{HOME}}/instance"

[control_plane]
enabled = false

[paths]
root_dir = "${{HOME}}/agent-root"
run_dir = "run"
state_dir = "state"
log_dir = "log"

[execution]
max_running_actions = 1
cancel_grace_ms = 5000
default_stdout_limit_bytes = 1048576
default_stderr_limit_bytes = 1048576
"#,
            ),
        )
        .expect("write config");

        let config = load_from_path(&config_path).expect("load config");

        assert_eq!(
            config.paths.root_dir,
            Path::new(&home).join("agent-root").display().to_string()
        );
        assert_eq!(
            config.paths.state_dir,
            Path::new(&home)
                .join("agent-root")
                .join("state")
                .display()
                .to_string()
        );
        assert_eq!(
            config.agent.instance_name.as_deref(),
            Some(format!("{home}/instance").as_str())
        );
    }
}
