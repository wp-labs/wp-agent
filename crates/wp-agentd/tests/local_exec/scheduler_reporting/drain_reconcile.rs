use std::fs;

use wp_agent_contracts::action_result::FinalStatus;
use wp_agent_shared::fs::read_json;
use wp_agentd::bootstrap;
use wp_agentd::local_exec::{LocalExecRequest, execute as execute_local};
use wp_agentd::reporting_pipeline;
use wp_agentd::scheduler::{
    DrainRequest, SchedulerRequest, drain_next_with_report, submit_local_plan,
};
use wp_agentd::state_store::{execution_queue, history, reporting, running};

use super::super::common::{sample_plan, temp_dir, test_exec_bin, write_exec_wrapper};

#[cfg(unix)]
#[test]
fn scheduler_drains_queue_and_prepares_report() {
    let root = temp_dir("spawn");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let submitted = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: sample_plan(),
    })
    .expect("submit local plan");

    let outcome = drain_next_with_report(&DrainRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin: test_exec_bin(&root),
        instance_id: "instance-001".to_string(),
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
    })
    .expect("drain queue")
    .expect("drain outcome");

    let queue_path = execution_queue::path_for(&state_dir);
    let queue_state = execution_queue::load_or_default(&queue_path).expect("queue state");
    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let reporting_state: reporting::ReportingState =
        read_json(&reporting_path).expect("read reporting state");
    let report_envelope_path =
        reporting_pipeline::envelope_path_for(&state_dir, &submitted.execution_id);
    let report_envelope: wp_agent_contracts::gateway::ReportActionResult =
        read_json(&report_envelope_path).expect("read report envelope");

    assert!(queue_state.items.is_empty());
    assert!(!running_path.exists());
    assert_eq!(
        outcome.report.final_status,
        wp_agent_contracts::action_result::FinalStatus::Succeeded
    );
    assert_eq!(outcome.report.result.final_status, FinalStatus::Succeeded);
    assert_eq!(reporting_state.final_state, "succeeded");
    assert_eq!(reporting_state.plan_digest, submitted.plan_digest);
    assert_eq!(report_envelope.execution_id, submitted.execution_id);
    assert_eq!(report_envelope.action_id, "act_001");
    assert_eq!(report_envelope.result.final_status, FinalStatus::Succeeded);
    assert_eq!(
        report_envelope.result_attestation.issued_by,
        "dev-placeholder:agent-001"
    );
    assert!(
        report_envelope
            .result_attestation
            .signature
            .starts_with("dev-placeholder-signature:agent-001:")
    );
    assert!(reporting_state.result_digest.is_some());
    assert!(reporting_state.result_signature.is_some());
    assert_eq!(reporting_state.report_attempt, 0);
    assert!(reporting_state.last_report_at.is_none());
    assert!(reporting_state.last_report_error.is_none());
    assert_eq!(report_envelope.report_attempt, 1);
    assert_eq!(
        report_envelope.report_id,
        format!("rep_{}_1", submitted.execution_id)
    );
    assert!(report_envelope_path.exists());
}

#[cfg(unix)]
#[test]
fn drain_reconciles_existing_result_without_rerun() {
    let root = temp_dir("drain-reconcile-result");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let plan = sample_plan();
    let submitted = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: plan.clone(),
    })
    .expect("submit local plan");

    execute_local(&LocalExecRequest {
        execution_id: submitted.execution_id.clone(),
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin: test_exec_bin(&root),
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
        plan_digest: submitted.plan_digest.clone(),
        request_id: "req_001".to_string(),
        plan: plan.clone(),
    })
    .expect("seed local exec result");

    let rerun_flag = root.join("rerun.flag");
    let fail_on_rerun = write_exec_wrapper(
        &root,
        &format!("echo rerun > \"{}\"\nexit 99", rerun_flag.display()),
    );

    let outcome = drain_next_with_report(&DrainRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin: fail_on_rerun,
        instance_id: "instance-001".to_string(),
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
    })
    .expect("reconcile queued execution")
    .expect("drain outcome");

    let queue_path = execution_queue::path_for(&state_dir);
    let queue_state = execution_queue::load_or_default(&queue_path).expect("queue state");
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);
    let reporting_state: reporting::ReportingState =
        read_json(&reporting_path).expect("read reporting state");

    assert!(queue_state.items.is_empty());
    assert!(!running_path.exists());
    assert!(!rerun_flag.exists());
    assert_eq!(outcome.report.result.final_status, FinalStatus::Succeeded);
    assert_eq!(reporting_state.final_state, "succeeded");
}

#[cfg(unix)]
#[test]
fn drain_rebuilds_reporting_state_when_envelope_exists_without_state() {
    let root = temp_dir("drain-rebuild-reporting-state");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let plan = sample_plan();
    let submitted = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: plan.clone(),
    })
    .expect("submit local plan");

    let local_outcome = execute_local(&LocalExecRequest {
        execution_id: submitted.execution_id.clone(),
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin: test_exec_bin(&root),
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
        plan_digest: submitted.plan_digest.clone(),
        request_id: "req_001".to_string(),
        plan: plan.clone(),
    })
    .expect("seed local exec result");

    reporting_pipeline::prepare_local_report(reporting_pipeline::ReportingRequest {
        state_dir: &state_dir,
        execution_id: &submitted.execution_id,
        action_id: "act_001",
        request_id: "req_001",
        plan_digest: &submitted.plan_digest,
        agent_id: &plan.target.agent_id,
        instance_id: "instance-001",
        final_state: "succeeded",
        result_path: &local_outcome.workdir.join("result.json"),
        result: &local_outcome.result,
    })
    .expect("prepare local report");

    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);
    let envelope_path = reporting_pipeline::envelope_path_for(&state_dir, &submitted.execution_id);
    fs::remove_file(&reporting_path).expect("remove reporting state");
    assert!(envelope_path.exists());

    let rerun_flag = root.join("rerun.flag");
    let fail_on_rerun = write_exec_wrapper(
        &root,
        &format!("echo rerun > \"{}\"\nexit 99", rerun_flag.display()),
    );

    let outcome = drain_next_with_report(&DrainRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin: fail_on_rerun,
        instance_id: "instance-001".to_string(),
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
    })
    .expect("reconcile queued execution")
    .expect("drain outcome");

    let queue_state =
        execution_queue::load_or_default(&execution_queue::path_for(&state_dir)).expect("queue");
    let reporting_state: reporting::ReportingState =
        read_json(&reporting_path).expect("read rebuilt reporting state");

    assert!(!rerun_flag.exists());
    assert!(queue_state.items.is_empty());
    assert_eq!(outcome.report.result.final_status, FinalStatus::Succeeded);
    assert_eq!(reporting_state.final_state, "succeeded");
}

#[cfg(unix)]
#[test]
fn drain_rebuilds_corrupt_reporting_state_without_rerun() {
    let root = temp_dir("drain-rebuild-corrupt-reporting-state");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let plan = sample_plan();
    let submitted = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: plan.clone(),
    })
    .expect("submit local plan");

    let local_outcome = execute_local(&LocalExecRequest {
        execution_id: submitted.execution_id.clone(),
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin: test_exec_bin(&root),
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
        plan_digest: submitted.plan_digest.clone(),
        request_id: "req_001".to_string(),
        plan: plan.clone(),
    })
    .expect("seed local exec result");

    reporting_pipeline::prepare_local_report(reporting_pipeline::ReportingRequest {
        state_dir: &state_dir,
        execution_id: &submitted.execution_id,
        action_id: "act_001",
        request_id: "req_001",
        plan_digest: &submitted.plan_digest,
        agent_id: &plan.target.agent_id,
        instance_id: "instance-001",
        final_state: "succeeded",
        result_path: &local_outcome.workdir.join("result.json"),
        result: &local_outcome.result,
    })
    .expect("prepare local report");

    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);
    fs::write(&reporting_path, "{bad json").expect("corrupt reporting state");

    let rerun_flag = root.join("rerun.flag");
    let fail_on_rerun = write_exec_wrapper(
        &root,
        &format!("echo rerun > \"{}\"\nexit 99", rerun_flag.display()),
    );

    let outcome = drain_next_with_report(&DrainRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin: fail_on_rerun,
        instance_id: "instance-001".to_string(),
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
    })
    .expect("reconcile queued execution")
    .expect("drain outcome");

    let queue_state =
        execution_queue::load_or_default(&execution_queue::path_for(&state_dir)).expect("queue");
    let rebuilt_reporting_state: reporting::ReportingState =
        read_json(&reporting_path).expect("read rebuilt reporting state");

    assert!(!rerun_flag.exists());
    assert!(queue_state.items.is_empty());
    assert_eq!(outcome.report.result.final_status, FinalStatus::Succeeded);
    assert_eq!(rebuilt_reporting_state.final_state, "succeeded");
    assert_eq!(rebuilt_reporting_state.report_attempt, 0);
    assert!(rebuilt_reporting_state.last_report_at.is_none());
}

#[cfg(unix)]
#[test]
fn drain_rebuilds_corrupt_envelope_without_quarantining_execution() {
    let root = temp_dir("drain-rebuild-corrupt-envelope");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let plan = sample_plan();
    let submitted = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: plan.clone(),
    })
    .expect("submit local plan");

    let local_outcome = execute_local(&LocalExecRequest {
        execution_id: submitted.execution_id.clone(),
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin: test_exec_bin(&root),
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
        plan_digest: submitted.plan_digest.clone(),
        request_id: "req_001".to_string(),
        plan: plan.clone(),
    })
    .expect("seed local exec result");

    reporting_pipeline::prepare_local_report(reporting_pipeline::ReportingRequest {
        state_dir: &state_dir,
        execution_id: &submitted.execution_id,
        action_id: "act_001",
        request_id: "req_001",
        plan_digest: &submitted.plan_digest,
        agent_id: &plan.target.agent_id,
        instance_id: "instance-001",
        final_state: "succeeded",
        result_path: &local_outcome.workdir.join("result.json"),
        result: &local_outcome.result,
    })
    .expect("prepare local report");

    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);
    let envelope_path = reporting_pipeline::envelope_path_for(&state_dir, &submitted.execution_id);
    let mut reporting_state: reporting::ReportingState =
        read_json(&reporting_path).expect("read reporting state");
    reporting_state.report_attempt = 2;
    reporting_state.last_report_at = Some("2026-04-12T10:00:09Z".to_string());
    reporting_state.last_report_error = Some("control plane unavailable".to_string());
    reporting::store(&reporting_path, &reporting_state).expect("store reporting state");
    fs::write(&envelope_path, "{bad json").expect("corrupt envelope");

    let rerun_flag = root.join("rerun.flag");
    let fail_on_rerun = write_exec_wrapper(
        &root,
        &format!("echo rerun > \"{}\"\nexit 99", rerun_flag.display()),
    );

    let outcome = drain_next_with_report(&DrainRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin: fail_on_rerun,
        instance_id: "instance-001".to_string(),
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
    })
    .expect("reconcile queued execution")
    .expect("drain outcome");

    let queue_state =
        execution_queue::load_or_default(&execution_queue::path_for(&state_dir)).expect("queue");
    let rebuilt_reporting_state: reporting::ReportingState =
        read_json(&reporting_path).expect("read reporting state");
    let rebuilt_envelope: wp_agent_contracts::gateway::ReportActionResult =
        read_json(&envelope_path).expect("read rebuilt envelope");
    let quarantine_path = history::path_for(&state_dir, &submitted.execution_id);

    assert!(!rerun_flag.exists());
    assert!(queue_state.items.is_empty());
    assert_eq!(rebuilt_reporting_state.final_state, "succeeded");
    assert_eq!(rebuilt_reporting_state.report_attempt, 2);
    assert_eq!(
        rebuilt_reporting_state.last_report_at.as_deref(),
        Some("2026-04-12T10:00:09Z")
    );
    assert_eq!(
        rebuilt_reporting_state.last_report_error.as_deref(),
        Some("control plane unavailable")
    );
    assert_eq!(outcome.report.result.final_status, FinalStatus::Succeeded);
    assert_eq!(rebuilt_envelope.report_attempt, 3);
    assert_eq!(
        rebuilt_envelope.report_id,
        format!("rep_{}_3", submitted.execution_id)
    );
    assert_eq!(rebuilt_envelope.result.final_status, FinalStatus::Succeeded);
    assert!(!quarantine_path.exists());
}
