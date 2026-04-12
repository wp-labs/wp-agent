//! File-input checkpoint state contract types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogStateContract {
    pub schema_version: String,
    pub input_id: String,
    pub updated_at: String,
    pub files: Vec<TrackedFileCheckpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedFileCheckpoint {
    pub file_id: String,
    pub path: String,
    pub checkpoint_offset: u64,
}
