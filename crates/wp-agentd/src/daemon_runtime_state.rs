use std::fs;
use std::io;
use std::path::Path;

use wp_agent_contracts::agent_config::AgentConfigContract;
use wp_agent_shared::paths::REPORT_ENVELOPE_SUFFIX;

use super::telemetry_support::{TelemetryFailure, TelemetryFailureKind};

pub(super) fn emit_telemetry_failures(failures: &[TelemetryFailure]) {
    for failure in failures {
        match failure.kind {
            TelemetryFailureKind::MissingInput => eprintln!(
                "telemetry input missing input_id={} path={}",
                failure.input_id, failure.path
            ),
            TelemetryFailureKind::ProcessingFailed => eprintln!(
                "telemetry input failed input_id={} path={} err={}",
                failure.input_id, failure.path, failure.detail
            ),
        }
    }
}

pub(super) fn count_running_entries(state_dir: &Path) -> io::Result<usize> {
    let running_dir = state_dir.join("running");
    if !running_dir.exists() {
        return Ok(0);
    }

    let mut count = 0usize;
    for entry in fs::read_dir(running_dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|ext| ext.to_str()) == Some("json") {
            count += 1;
        }
    }
    Ok(count)
}

pub(super) fn count_reporting_entries(state_dir: &Path) -> io::Result<usize> {
    let reporting_dir = state_dir.join("reporting");
    if !reporting_dir.exists() {
        return Ok(0);
    }

    let mut count = 0usize;
    for entry in fs::read_dir(reporting_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if file_name.ends_with(REPORT_ENVELOPE_SUFFIX) {
            continue;
        }
        count += 1;
    }
    Ok(count)
}

pub(super) fn instance_id(config: &AgentConfigContract) -> String {
    config
        .agent
        .instance_name
        .clone()
        .unwrap_or_else(|| "local-instance".to_string())
}
