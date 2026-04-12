//! `agent_runtime.json` store.

use std::io;
use std::path::{Path, PathBuf};

use wp_agent_contracts::state_exec::AgentRuntimeState;
use wp_agent_contracts::state_exec::RuntimeMode;
use wp_agent_shared::fs::{read_json, write_json_atomic};
use wp_agent_shared::paths::AGENT_RUNTIME_FILE;
use wp_agent_shared::time::now_rfc3339;

pub fn load_default() -> AgentRuntimeState {
    AgentRuntimeState::new(
        "local-agent".to_string(),
        "local-instance".to_string(),
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
