//! Structured record sinks for local `warp-parse` style output.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;

use wp_agent_contracts::telemetry_record::TelemetryRecordContract;
use wp_agent_shared::fs::ensure_parent;

pub trait RecordSink {
    fn write_records(&mut self, records: &[TelemetryRecordContract]) -> io::Result<()>;
}

#[derive(Debug, Clone)]
pub struct FileRecordSink {
    path: PathBuf,
}

impl FileRecordSink {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl RecordSink for FileRecordSink {
    fn write_records(&mut self, records: &[TelemetryRecordContract]) -> io::Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        ensure_parent(&self.path)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        for record in records {
            serde_json::to_writer(&mut file, record).map_err(io::Error::other)?;
            file.write_all(b"\n")?;
        }
        file.sync_all()?;
        Ok(())
    }
}
