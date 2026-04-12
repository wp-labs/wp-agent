//! `running/<execution_id>.json` store.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use wp_agent_contracts::SCHEMA_VERSION_V1;
use wp_agent_shared::fs::{read_json, write_json_atomic};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunningExecutionState {
    pub schema_version: String,
    pub execution_id: String,
    pub action_id: String,
    pub plan_digest: String,
    pub request_id: String,
    pub state: String,
    pub workdir: String,
    pub pid: Option<u32>,
    pub started_at: String,
    pub deadline_at: Option<String>,
    pub current_step_id: Option<String>,
    pub attempt: Option<u32>,
    pub cancel_requested_at: Option<String>,
    pub kill_requested_at: Option<String>,
    pub updated_at: String,
}

impl RunningExecutionState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        execution_id: String,
        action_id: String,
        plan_digest: String,
        request_id: String,
        state: String,
        workdir: String,
        pid: Option<u32>,
        started_at: String,
        deadline_at: Option<String>,
        current_step_id: Option<String>,
        attempt: Option<u32>,
        cancel_requested_at: Option<String>,
        kill_requested_at: Option<String>,
        updated_at: String,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            execution_id,
            action_id,
            plan_digest,
            request_id,
            state,
            workdir,
            pid,
            started_at,
            deadline_at,
            current_step_id,
            attempt,
            cancel_requested_at,
            kill_requested_at,
            updated_at,
        }
    }
}

pub fn path_for(state_dir: &Path, execution_id: &str) -> PathBuf {
    state_dir
        .join("running")
        .join(format!("{execution_id}.json"))
}

pub fn load(path: &Path) -> io::Result<RunningExecutionState> {
    read_json(path)
}

pub fn store(path: &Path, state: &RunningExecutionState) -> io::Result<()> {
    write_json_atomic(path, state)
}

pub fn remove(path: &Path) -> io::Result<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
