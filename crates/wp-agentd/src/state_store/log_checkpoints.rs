//! `state/logs/file_inputs/*/checkpoints.json` store.

use std::io;
use std::path::{Path, PathBuf};

use wp_agent_contracts::state_logs::LogStateContract;
use wp_agent_shared::fs::{ensure_parent, read_json, write_json_atomic};
use wp_agent_shared::time::now_rfc3339;

pub fn load_or_default(input_id: &str) -> LogStateContract {
    LogStateContract::new(input_id.to_string(), now_rfc3339())
}

pub fn path_for(state_dir: &Path, input_id: &str) -> PathBuf {
    state_dir
        .join("logs")
        .join("file_inputs")
        .join(input_id)
        .join("checkpoints.json")
}

pub fn load_or_default_from_path(path: &Path, input_id: &str) -> io::Result<LogStateContract> {
    if path.exists() {
        read_json(path)
    } else {
        Ok(load_or_default(input_id))
    }
}

pub fn store(path: &Path, state: &LogStateContract) -> io::Result<()> {
    ensure_parent(path)?;
    write_json_atomic(path, state)
}

pub fn ensure_parent_dir(path: &Path) -> io::Result<()> {
    ensure_parent(path)?;
    Ok(())
}
