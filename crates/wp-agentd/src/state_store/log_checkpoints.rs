//! `state/logs/file_inputs/*/checkpoints.json` store.

use std::io;
use std::path::{Path, PathBuf};

use wp_agent_shared::fs::{ensure_parent, read_json, write_json_atomic};
use wp_agent_shared::time::now_rfc3339;

use crate::state_store::log_checkpoint_state::LogCheckpointState;

pub(crate) fn load_or_default(input_id: &str) -> LogCheckpointState {
    LogCheckpointState::new(input_id.to_string(), now_rfc3339())
}

pub fn path_for(state_dir: &Path, input_id: &str) -> PathBuf {
    state_dir
        .join("logs")
        .join("file_inputs")
        .join(input_id)
        .join("checkpoints.json")
}

pub(crate) fn load_or_default_from_path(
    path: &Path,
    input_id: &str,
) -> io::Result<LogCheckpointState> {
    if path.exists() {
        read_json(path)
    } else {
        Ok(load_or_default(input_id))
    }
}

pub(crate) fn store(path: &Path, state: &LogCheckpointState) -> io::Result<()> {
    ensure_parent(path)?;
    write_json_atomic(path, state)
}
