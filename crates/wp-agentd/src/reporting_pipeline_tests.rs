use super::{
    LocalReportIssue, PreparedReportOrigin, ReportingRequest, ensure_local_report,
    prepare_local_report,
};
use crate::state_store::reporting;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use wp_agent_contracts::action_result::{
    ActionResultContract, FinalStatus, StepActionRecord, StepStatus,
};
use wp_agent_shared::fs::write_json_atomic;

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("wp-agentd-reporting-{name}-{suffix}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn sample_result() -> ActionResultContract {
    let mut result = ActionResultContract::new(
        "act_001".to_string(),
        "exec_001".to_string(),
        FinalStatus::Succeeded,
    );
    result.request_id = Some("req_001".to_string());
    result.step_records = vec![StepActionRecord {
        step_id: "step_collect".to_string(),
        attempt: 1,
        op: Some("process.list".to_string()),
        status: StepStatus::Succeeded,
        started_at: "2026-04-12T10:00:00Z".to_string(),
        finished_at: Some("2026-04-12T10:00:01Z".to_string()),
        duration_ms: Some(1),
        error_code: None,
        stdout_summary: None,
        stderr_summary: None,
        resource_usage: None,
    }];
    result.started_at = Some("2026-04-12T10:00:00Z".to_string());
    result.finished_at = Some("2026-04-12T10:00:01Z".to_string());
    result
}

fn sample_request<'a>(
    state_dir: &'a std::path::Path,
    result_path: &'a std::path::Path,
    result: &'a ActionResultContract,
) -> ReportingRequest<'a> {
    ReportingRequest {
        state_dir,
        execution_id: "exec_001",
        action_id: "act_001",
        request_id: "req_001",
        plan_digest: "digest_001",
        agent_id: "agent_001",
        instance_id: "instance_001",
        final_state: "succeeded",
        result_path,
        result,
    }
}

#[test]
fn prepare_local_report_marks_new_report_origin() {
    let root = temp_dir("new-report");
    let state_dir = root.join("state");
    let result_path = root
        .join("run")
        .join("actions")
        .join("exec_001")
        .join("result.json");
    let result = sample_result();
    write_json_atomic(&result_path, &result).expect("write result");

    let prepared =
        prepare_local_report(sample_request(&state_dir, &result_path, &result)).expect("prepare");

    assert_eq!(
        prepared.origin,
        PreparedReportOrigin::Prepared(LocalReportIssue::NewReport)
    );
}

#[test]
fn ensure_local_report_marks_missing_state_repair() {
    let root = temp_dir("missing-state");
    let state_dir = root.join("state");
    let result_path = root
        .join("run")
        .join("actions")
        .join("exec_001")
        .join("result.json");
    let result = sample_result();
    write_json_atomic(&result_path, &result).expect("write result");

    let prepared =
        ensure_local_report(sample_request(&state_dir, &result_path, &result)).expect("ensure");

    assert_eq!(
        prepared.origin,
        PreparedReportOrigin::Prepared(LocalReportIssue::MissingState)
    );
}

#[test]
fn ensure_local_report_marks_missing_envelope_repair() {
    let root = temp_dir("missing-envelope");
    let state_dir = root.join("state");
    let result_path = root
        .join("run")
        .join("actions")
        .join("exec_001")
        .join("result.json");
    let result = sample_result();
    write_json_atomic(&result_path, &result).expect("write result");

    let initial =
        prepare_local_report(sample_request(&state_dir, &result_path, &result)).expect("prepare");
    fs::remove_file(&initial.envelope_path).expect("remove envelope");

    let repaired =
        ensure_local_report(sample_request(&state_dir, &result_path, &result)).expect("repair");

    assert_eq!(
        repaired.origin,
        PreparedReportOrigin::EnvelopeRebuilt(LocalReportIssue::MissingEnvelope)
    );
    assert_eq!(repaired.state.result_digest, initial.state.result_digest);
    assert_eq!(
        repaired.state.result_signature,
        initial.state.result_signature
    );
}

#[test]
fn ensure_local_report_marks_corrupt_state_repair() {
    let root = temp_dir("corrupt-state");
    let state_dir = root.join("state");
    let result_path = root
        .join("run")
        .join("actions")
        .join("exec_001")
        .join("result.json");
    let result = sample_result();
    write_json_atomic(&result_path, &result).expect("write result");

    let prepared =
        prepare_local_report(sample_request(&state_dir, &result_path, &result)).expect("prepare");
    fs::write(
        reporting::path_for(&state_dir, "exec_001"),
        "{ not valid json",
    )
    .expect("corrupt state");

    let repaired =
        ensure_local_report(sample_request(&state_dir, &result_path, &result)).expect("repair");

    assert_eq!(
        repaired.origin,
        PreparedReportOrigin::Prepared(LocalReportIssue::CorruptState)
    );
    assert_eq!(repaired.envelope.report_id, prepared.envelope.report_id);
    assert_eq!(
        reporting::load(&reporting::path_for(&state_dir, "exec_001"))
            .expect("load repaired state")
            .result_path,
        repaired.state.result_path
    );
}
