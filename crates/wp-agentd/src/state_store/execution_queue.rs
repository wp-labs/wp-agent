//! `execution_queue.json` store.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use wp_agent_contracts::SCHEMA_VERSION_V1ALPHA1;
use wp_agent_shared::fs::{read_json, write_json_atomic};
use wp_agent_shared::paths::EXECUTION_QUEUE_FILE;
use wp_agent_shared::time::now_rfc3339;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionQueueState {
    pub schema_version: String,
    pub updated_at: String,
    pub items: Vec<ExecutionQueueItem>,
}

impl ExecutionQueueState {
    pub fn new() -> Self {
        Self {
            schema_version: SCHEMA_VERSION_V1ALPHA1.to_string(),
            updated_at: now_rfc3339(),
            items: Vec::new(),
        }
    }

    pub fn enqueue(&mut self, item: ExecutionQueueItem) {
        self.items.push(item);
        self.items.sort_by_key(|entry| entry.priority);
        self.updated_at = now_rfc3339();
    }

    pub fn remove(&mut self, execution_id: &str) {
        self.items.retain(|item| item.execution_id != execution_id);
        self.updated_at = now_rfc3339();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionQueueItem {
    pub execution_id: String,
    pub action_id: String,
    pub plan_digest: String,
    pub request_id: String,
    pub priority: u32,
    pub queued_at: String,
    pub deadline_at: Option<String>,
    pub cancelable: bool,
    pub risk_level: Option<String>,
}

impl ExecutionQueueItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        execution_id: String,
        action_id: String,
        plan_digest: String,
        request_id: String,
        priority: u32,
        queued_at: String,
        deadline_at: Option<String>,
        cancelable: bool,
        risk_level: Option<String>,
    ) -> Self {
        Self {
            execution_id,
            action_id,
            plan_digest,
            request_id,
            priority,
            queued_at,
            deadline_at,
            cancelable,
            risk_level,
        }
    }
}

pub fn path_for(state_dir: &Path) -> PathBuf {
    state_dir.join(EXECUTION_QUEUE_FILE)
}

pub fn load_or_default(path: &Path) -> io::Result<ExecutionQueueState> {
    if path.exists() {
        read_json(path)
    } else {
        Ok(ExecutionQueueState::new())
    }
}

pub fn store(path: &Path, state: &ExecutionQueueState) -> io::Result<()> {
    write_json_atomic(path, state)
}
