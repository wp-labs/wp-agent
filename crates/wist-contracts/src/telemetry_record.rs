//! Minimal structured telemetry record contract types.

use serde::{Deserialize, Serialize};

use crate::SCHEMA_VERSION_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryRecordContract {
    pub schema_version: String,
    pub signal_kind: String,
    pub observed_at: String,
    pub input_id: String,
    pub source_path: String,
    pub body: String,
    pub file_offset: u64,
    pub file_offset_end: u64,
}

impl TelemetryRecordContract {
    pub fn new_log(
        observed_at: String,
        input_id: String,
        source_path: String,
        body: String,
        file_offset: u64,
        file_offset_end: u64,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            signal_kind: "log".to_string(),
            observed_at,
            input_id,
            source_path,
            body,
            file_offset,
            file_offset_end,
        }
    }
}
