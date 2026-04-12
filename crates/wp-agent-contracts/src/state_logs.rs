//! File-input checkpoint state contract types.

use serde::{Deserialize, Serialize};

use crate::SCHEMA_VERSION_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogStateContract {
    pub schema_version: String,
    pub input_id: String,
    pub updated_at: String,
    pub files: Vec<TrackedFileCheckpoint>,
}

impl LogStateContract {
    pub fn new(input_id: String, updated_at: String) -> Self {
        Self {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            input_id,
            updated_at,
            files: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrackedFileCheckpoint {
    pub file_id: String,
    pub path: String,
    pub device_id: Option<u64>,
    pub inode: Option<u64>,
    pub fingerprint: Option<String>,
    pub checkpoint_offset: u64,
    pub last_size: Option<u64>,
    pub last_read_at: Option<String>,
    pub last_commit_point_at: Option<String>,
    pub rotated_from_path: Option<String>,
}
