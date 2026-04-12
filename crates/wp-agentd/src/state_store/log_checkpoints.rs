//! `state/logs/file_inputs/*/checkpoints.json` store.

use std::fs;
use std::io;
use std::path::Path;

use wp_agent_contracts::state_logs::LogStateContract;

pub fn load_or_default(input_id: &str) -> LogStateContract {
    LogStateContract {
        schema_version: "v1alpha1".to_string(),
        input_id: input_id.to_string(),
        updated_at: "1970-01-01T00:00:00Z".to_string(),
        files: Vec::new(),
    }
}

pub fn ensure_parent(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
