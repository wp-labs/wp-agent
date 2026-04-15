//! `agent_runtime.json` store.

use std::io;
use std::path::{Path, PathBuf};

use warp_insight_contracts::state_exec::AgentRuntimeState;
use warp_insight_contracts::state_exec::RuntimeMode;
use warp_insight_shared::fs::{read_json, write_json_atomic};
use warp_insight_shared::paths::AGENT_RUNTIME_FILE;
use warp_insight_shared::time::now_rfc3339;

pub fn load_default() -> AgentRuntimeState {
    AgentRuntimeState::new(
        "local-agent".to_string(),
        default_instance_id(),
        env!("CARGO_PKG_VERSION").to_string(),
        RuntimeMode::Normal,
        now_rfc3339(),
    )
}

pub fn path_for(state_dir: &Path) -> PathBuf {
    state_dir.join(AGENT_RUNTIME_FILE)
}

pub fn load_or_default(path: &Path) -> io::Result<AgentRuntimeState> {
    if path.exists() {
        read_json(path)
    } else {
        Ok(load_default())
    }
}

pub fn store(path: &Path, state: &AgentRuntimeState) -> io::Result<()> {
    write_json_atomic(path, state)
}

fn default_instance_id() -> String {
    default_instance_id_from_sources(
        std::env::var("HOSTNAME").ok().as_deref(),
        std::env::var("COMPUTERNAME").ok().as_deref(),
        hostname_from_file().as_deref(),
    )
}

fn default_instance_id_from_sources(
    hostname_env: Option<&str>,
    computername_env: Option<&str>,
    hostname_file: Option<&str>,
) -> String {
    hostname_env
        .or(computername_env)
        .or(hostname_file)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("local-instance")
        .to_string()
}

#[cfg(unix)]
fn hostname_from_file() -> Option<String> {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(not(unix))]
fn hostname_from_file() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::default_instance_id_from_sources;

    #[test]
    fn default_instance_id_prefers_hostname_env() {
        assert_eq!(
            default_instance_id_from_sources(Some("host-a"), Some("pc-a"), Some("file-a")),
            "host-a"
        );
    }

    #[test]
    fn default_instance_id_falls_back_to_hostname_file() {
        assert_eq!(
            default_instance_id_from_sources(None, None, Some("file-a")),
            "file-a"
        );
    }

    #[test]
    fn default_instance_id_uses_local_instance_when_all_sources_missing() {
        assert_eq!(
            default_instance_id_from_sources(None, None, None),
            "local-instance"
        );
    }
}
