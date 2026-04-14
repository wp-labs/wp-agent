use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

use wp_agent_contracts::agent_config::AgentConfigContract;
use wp_agent_shared::paths::REPORT_ENVELOPE_SUFFIX;

use super::telemetry_support::{TelemetryFailure, TelemetryFailureKind};

pub(super) fn emit_telemetry_failures(failures: &[TelemetryFailure]) {
    for failure in failures {
        emit_telemetry_failure(failure);
    }
}

pub(super) fn emit_telemetry_failure(failure: &TelemetryFailure) {
    match failure.kind {
        TelemetryFailureKind::MissingInput => eprintln!(
            "telemetry input missing input_id={} path={}",
            failure.input_id, failure.path
        ),
        TelemetryFailureKind::ProcessingFailed => eprintln!(
            "telemetry input failed input_id={} path={} err={}",
            failure.input_id, failure.path, failure.detail
        ),
        TelemetryFailureKind::InvalidOutput => eprintln!(
            "telemetry output invalid input_id={} path={} err={}",
            failure.input_id, failure.path, failure.detail
        ),
    }
}

pub(super) fn failure_signatures(failures: &[TelemetryFailure]) -> BTreeSet<String> {
    failures.iter().map(failure_signature).collect()
}

pub(super) fn filter_new_failures<'a>(
    failures: &'a [TelemetryFailure],
    previous: &BTreeSet<String>,
) -> Vec<&'a TelemetryFailure> {
    failures
        .iter()
        .filter(|failure| !previous.contains(&failure_signature(failure)))
        .collect()
}

fn failure_signature(failure: &TelemetryFailure) -> String {
    format!(
        "{:?}|{}|{}|{}",
        failure.kind, failure.input_id, failure.path, failure.detail
    )
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{TelemetryFailure, TelemetryFailureKind, failure_signatures, filter_new_failures};

    fn failure(
        kind: TelemetryFailureKind,
        input_id: &str,
        path: &str,
        detail: &str,
    ) -> TelemetryFailure {
        TelemetryFailure {
            kind,
            input_id: input_id.to_string(),
            path: path.to_string(),
            detail: detail.to_string(),
        }
    }

    #[test]
    fn failure_signatures_use_all_failure_identity_fields() {
        let signatures = failure_signatures(&[
            failure(
                TelemetryFailureKind::MissingInput,
                "app",
                "/tmp/a.log",
                "missing",
            ),
            failure(
                TelemetryFailureKind::MissingInput,
                "app",
                "/tmp/a.log",
                "missing",
            ),
            failure(
                TelemetryFailureKind::ProcessingFailed,
                "app",
                "/tmp/a.log",
                "failed",
            ),
        ]);

        assert_eq!(
            signatures,
            BTreeSet::from([
                "MissingInput|app|/tmp/a.log|missing".to_string(),
                "ProcessingFailed|app|/tmp/a.log|failed".to_string(),
            ])
        );
    }

    #[test]
    fn filter_new_failures_only_returns_entries_missing_from_previous_snapshot() {
        let failures = vec![
            failure(
                TelemetryFailureKind::MissingInput,
                "app",
                "/tmp/a.log",
                "missing",
            ),
            failure(
                TelemetryFailureKind::ProcessingFailed,
                "app",
                "/tmp/a.log",
                "failed",
            ),
        ];
        let previous = BTreeSet::from(["MissingInput|app|/tmp/a.log|missing".to_string()]);

        let filtered = filter_new_failures(&failures, &previous);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].detail, "failed");
    }
}
