use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(unix)]
use std::{os::unix::fs::PermissionsExt, path::Path};

use time::Duration as TimeDuration;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use wp_agent_contracts::action_plan::{
    ActionPlanConstraints, ActionPlanContract, ActionPlanMeta, ActionPlanProgram, ActionPlanStep,
    ActionPlanTarget, ApprovalMode, RiskLevel,
};
use wp_agent_contracts::action_result::FinalStatus;
use wp_agent_contracts::agent_config::{
    AgentConfigContract, AgentSection, ControlPlaneSection, ExecutionSection, PathsSection,
};
use wp_agent_shared::fs::{read_json, write_json_atomic};
use wp_agentd::bootstrap;
use wp_agentd::daemon;
use wp_agentd::local_exec::{LocalExecRequest, execute as execute_local};
use wp_agentd::reporting_pipeline;
use wp_agentd::scheduler::{
    DrainRequest, SchedulerRequest, drain_next_with_report, submit_local_plan,
};
use wp_agentd::state_store::{execution_queue, history, reporting, running};

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("wp-agentd-local-exec-{name}-{suffix}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[cfg(unix)]
fn test_exec_bin(root: &Path) -> PathBuf {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");
    let wrapper = root.join("wp-agent-exec-wrapper.sh");
    fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\ncd \"{}\"\nexec cargo run -q -p wp-agent-exec -- \"$@\"\n",
            workspace_root.display()
        ),
    )
    .expect("write wrapper");
    let mut perms = fs::metadata(&wrapper)
        .expect("wrapper metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&wrapper, perms).expect("set wrapper permissions");
    wrapper
}

#[cfg(unix)]
fn write_exec_wrapper(root: &Path, body: &str) -> PathBuf {
    let wrapper = root.join("wp-agent-exec-test-wrapper.sh");
    fs::write(&wrapper, format!("#!/bin/sh\n{body}\n")).expect("write wrapper");
    let mut perms = fs::metadata(&wrapper)
        .expect("wrapper metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&wrapper, perms).expect("set wrapper permissions");
    wrapper
}

fn sample_plan() -> ActionPlanContract {
    ActionPlanContract::new(
        ActionPlanMeta {
            action_id: "act_001".to_string(),
            request_id: "req_001".to_string(),
            template_id: None,
            tenant_id: "tenant_a".to_string(),
            environment_id: "prod-cn".to_string(),
            plan_version: 1,
            compiled_at: "2026-04-12T10:00:00Z".to_string(),
            expires_at: "2026-04-12T10:05:00Z".to_string(),
        },
        ActionPlanTarget {
            agent_id: "agent-001".to_string(),
            instance_id: Some("instance-001".to_string()),
            node_id: "node-001".to_string(),
            host_name: None,
            platform: "linux".to_string(),
            arch: "amd64".to_string(),
            selectors: Default::default(),
        },
        ActionPlanConstraints {
            risk_level: RiskLevel::R1,
            approval_ref: None,
            approval_mode: ApprovalMode::Required,
            requested_by: "alice@example.com".to_string(),
            reason: None,
            max_total_duration_ms: 30_000,
            step_timeout_default_ms: 10_000,
            execution_profile: "agent_exec_v1".to_string(),
            required_capabilities: vec!["process.list".to_string()],
        },
        ActionPlanProgram {
            entry: "step_collect".to_string(),
            steps: vec![ActionPlanStep {
                id: "step_collect".to_string(),
                kind: "invoke".to_string(),
                op: Some("process.list".to_string()),
            }],
        },
    )
}

fn sample_plan_with_ids(action_id: &str, request_id: &str) -> ActionPlanContract {
    let mut plan = sample_plan();
    plan.meta.action_id = action_id.to_string();
    plan.meta.request_id = request_id.to_string();
    plan
}

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
fn local_exec_synthesizes_timeout_result() {
    let root = temp_dir("timeout");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let mut plan = sample_plan();
    plan.constraints.max_total_duration_ms = 50;
    let exec_bin = write_exec_wrapper(&root, "sleep 1");

    let outcome = execute_local(&LocalExecRequest {
        execution_id: "exec_timeout".to_string(),
        run_dir,
        state_dir,
        exec_bin,
        cancel_grace_ms: 50,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
        plan_digest: "digest-timeout".to_string(),
        request_id: "req_001".to_string(),
        plan,
    })
    .expect("timeout local exec");

    let state_json: serde_json::Value =
        read_json(&outcome.workdir.join("state.json")).expect("read state json");
    assert_eq!(outcome.result.final_status, FinalStatus::TimedOut);
    assert_eq!(
        outcome.result.exit_reason.as_deref(),
        Some("agentd_total_timeout")
    );
    assert_eq!(state_json["state"], "timed_out");
}

#[cfg(unix)]
#[test]
fn local_exec_synthesizes_failed_result_on_abnormal_exit() {
    let root = temp_dir("abnormal-exit");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let exec_bin = write_exec_wrapper(&root, "exit 7");

    let outcome = execute_local(&LocalExecRequest {
        execution_id: "exec_fail".to_string(),
        run_dir,
        state_dir,
        exec_bin,
        cancel_grace_ms: 50,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
        plan_digest: "digest-fail".to_string(),
        request_id: "req_001".to_string(),
        plan: sample_plan(),
    })
    .expect("abnormal exit local exec");

    let state_json: serde_json::Value =
        read_json(&outcome.workdir.join("state.json")).expect("read state json");
    assert_eq!(outcome.result.final_status, FinalStatus::Failed);
    assert_eq!(outcome.result.exit_reason.as_deref(), Some("exec_exit_7"));
    assert_eq!(state_json["state"], "failed");
}

#[cfg(unix)]
#[test]
fn recovery_turns_incomplete_running_state_into_reporting() {
    let root = temp_dir("recover");
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

    let workdir = run_dir.join("actions").join(&submitted.execution_id);
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let running_state = running::RunningExecutionState::new(
        submitted.execution_id.clone(),
        "act_001".to_string(),
        submitted.plan_digest.clone(),
        "req_001".to_string(),
        "running".to_string(),
        workdir.display().to_string(),
        Some(4242),
        None,
        "2026-04-12T10:00:00Z".to_string(),
        Some("2026-04-12T10:05:00Z".to_string()),
        Some("step_collect".to_string()),
        Some(1),
        None,
        None,
        "2026-04-12T10:00:01Z".to_string(),
    );
    write_json_atomic(&running_path, &running_state).expect("write running state");

    daemon::recover_incomplete_executions(&state_dir, "instance-001").expect("recover");

    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);
    let report_envelope_path =
        reporting_pipeline::envelope_path_for(&state_dir, &submitted.execution_id);
    let reporting_state: reporting::ReportingState =
        read_json(&reporting_path).expect("read reporting state");
    let report_envelope: wp_agent_contracts::gateway::ReportActionResult =
        read_json(&report_envelope_path).expect("read report envelope");

    assert!(!running_path.exists());
    assert_eq!(reporting_state.final_state, "failed");
    assert_eq!(reporting_state.report_attempt, 0);
    assert!(reporting_state.last_report_at.is_none());
    assert_eq!(
        report_envelope.result.exit_reason.as_deref(),
        Some("agentd_recovered_incomplete_execution")
    );
    assert_eq!(
        report_envelope.report_id,
        format!("rep_{}_1", submitted.execution_id)
    );
}

#[cfg(unix)]
#[test]
fn recovery_reuses_reporting_state_and_rebuilds_report_attempt() {
    let root = temp_dir("recover-rebuild-envelope");
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

    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);
    let envelope_path = reporting_pipeline::envelope_path_for(&state_dir, &submitted.execution_id);
    let mut reporting_state: reporting::ReportingState =
        read_json(&reporting_path).expect("read reporting state");
    reporting_state.report_attempt = 2;
    reporting_state.last_report_at = Some("2026-04-12T10:00:09Z".to_string());
    reporting_state.last_report_error = Some("cp unavailable".to_string());
    reporting::store(&reporting_path, &reporting_state).expect("store reporting state");
    fs::write(&envelope_path, "{bad json").expect("corrupt envelope");

    daemon::recover_incomplete_executions(&state_dir, "instance-001").expect("recover");

    let rebuilt_reporting_state: reporting::ReportingState =
        read_json(&reporting_path).expect("read rebuilt reporting state");
    let rebuilt_envelope: wp_agent_contracts::gateway::ReportActionResult =
        read_json(&envelope_path).expect("read rebuilt envelope");

    assert!(!running_path.exists());
    assert_eq!(rebuilt_reporting_state.report_attempt, 2);
    assert_eq!(
        rebuilt_reporting_state.last_report_at.as_deref(),
        Some("2026-04-12T10:00:09Z")
    );
    assert_eq!(
        rebuilt_reporting_state.last_report_error.as_deref(),
        Some("cp unavailable")
    );
    assert_eq!(rebuilt_envelope.report_attempt, 3);
    assert_eq!(
        rebuilt_envelope.report_id,
        format!("rep_{}_3", submitted.execution_id)
    );
}

#[cfg(unix)]
#[test]
fn recovery_quarantines_execution_when_report_preparation_fails() {
    let root = temp_dir("recover-report-prep-fail");
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

    let workdir = run_dir.join("actions").join(&submitted.execution_id);
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let running_state = running::RunningExecutionState::new(
        submitted.execution_id.clone(),
        "act_001".to_string(),
        submitted.plan_digest.clone(),
        "req_001".to_string(),
        "running".to_string(),
        workdir.display().to_string(),
        Some(4242),
        None,
        "2026-04-12T10:00:00Z".to_string(),
        Some("2026-04-12T10:05:00Z".to_string()),
        Some("step_collect".to_string()),
        Some(1),
        None,
        None,
        "2026-04-12T10:00:01Z".to_string(),
    );
    write_json_atomic(&running_path, &running_state).expect("write running state");

    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);
    let blocked_state_tmp_path = reporting_path.with_extension("tmp");
    fs::create_dir(&blocked_state_tmp_path).expect("block reporting state tmp path");

    daemon::recover_incomplete_executions(&state_dir, "instance-001").expect("recover");

    let quarantine_path = history::path_for(&state_dir, &submitted.execution_id);
    let quarantine: history::ExecutionHistoryRecord =
        read_json(&quarantine_path).expect("read quarantine record");
    let envelope_path = reporting_pipeline::envelope_path_for(&state_dir, &submitted.execution_id);

    assert!(!running_path.exists());
    assert!(!reporting_path.exists());
    assert!(!envelope_path.exists());
    assert_eq!(quarantine.state, "quarantined");
    assert_eq!(
        quarantine.plan_digest.as_deref(),
        Some(submitted.plan_digest.as_str())
    );
    assert!(
        quarantine
            .detail
            .contains("running execution report preparation failed")
    );
}

#[cfg(unix)]
#[test]
fn recovery_skips_execution_when_pid_is_still_alive() {
    let root = temp_dir("recover-live-pid");
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

    let workdir = run_dir.join("actions").join(&submitted.execution_id);
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);
    let running_state = running::RunningExecutionState::new(
        submitted.execution_id.clone(),
        "act_001".to_string(),
        submitted.plan_digest.clone(),
        "req_001".to_string(),
        "running".to_string(),
        workdir.display().to_string(),
        Some(std::process::id()),
        None,
        rfc3339_before_now(1),
        Some(rfc3339_after_now(60)),
        Some("step_collect".to_string()),
        Some(1),
        None,
        None,
        rfc3339_before_now(1),
    );
    write_json_atomic(&running_path, &running_state).expect("write running state");

    daemon::recover_incomplete_executions(&state_dir, "instance-001").expect("recover");

    assert!(running_path.exists());
    assert!(!reporting_path.exists());
}

#[cfg(unix)]
#[test]
fn submit_local_plan_does_not_enqueue_when_queue_store_fails() {
    let root = temp_dir("submit-queue-fail");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let mut perms = fs::metadata(&state_dir)
        .expect("state metadata")
        .permissions();
    perms.set_mode(0o500);
    fs::set_permissions(&state_dir, perms).expect("set state permissions");

    let submit = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: sample_plan(),
    });

    let mut restore = fs::metadata(&state_dir)
        .expect("state metadata after submit")
        .permissions();
    restore.set_mode(0o700);
    fs::set_permissions(&state_dir, restore).expect("restore state permissions");

    assert!(submit.is_err());

    let queue_path = execution_queue::path_for(&state_dir);
    let queue_state = execution_queue::load_or_default(&queue_path).expect("queue state");
    let actions_dir = run_dir.join("actions");
    let action_entries = fs::read_dir(&actions_dir)
        .expect("read actions dir")
        .count();

    assert!(queue_state.items.is_empty());
    assert_eq!(action_entries, 0);
}

#[cfg(unix)]
#[test]
fn submit_local_plan_rejects_duplicate_plan_in_queue() {
    let root = temp_dir("submit-duplicate-queue");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let plan = sample_plan();
    let first = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: plan.clone(),
    })
    .expect("submit first plan");

    let err = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan,
    })
    .expect_err("duplicate plan should be rejected");

    let queue_state =
        execution_queue::load_or_default(&execution_queue::path_for(&state_dir)).expect("queue");
    let action_entries = fs::read_dir(run_dir.join("actions"))
        .expect("read actions dir")
        .count();

    assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    assert!(err.to_string().contains(&first.execution_id));
    assert_eq!(queue_state.items.len(), 1);
    assert_eq!(queue_state.items[0].execution_id, first.execution_id);
    assert_eq!(action_entries, 1);
}

#[cfg(unix)]
#[test]
fn submit_local_plan_rejects_duplicate_plan_when_reporting_exists() {
    let root = temp_dir("submit-duplicate-reporting");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let plan = sample_plan();
    let first = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: plan.clone(),
    })
    .expect("submit first plan");

    let _outcome = drain_next_with_report(&DrainRequest {
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

    let err = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan,
    })
    .expect_err("duplicate plan should be rejected");

    let queue_state =
        execution_queue::load_or_default(&execution_queue::path_for(&state_dir)).expect("queue");
    let reporting_path = reporting::path_for(&state_dir, &first.execution_id);

    assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    assert!(err.to_string().contains(&first.execution_id));
    assert!(queue_state.items.is_empty());
    assert!(reporting_path.exists());
}

fn standalone_config(root: &std::path::Path) -> AgentConfigContract {
    AgentConfigContract::new(
        AgentSection {
            agent_id: Some("agent-001".to_string()),
            environment_id: Some("prod-cn".to_string()),
            instance_name: Some("instance-001".to_string()),
        },
        ControlPlaneSection {
            enabled: false,
            endpoint: None,
            tls_mode: None,
            auth_mode: None,
        },
        PathsSection {
            root_dir: root.display().to_string(),
            run_dir: root.join("run").display().to_string(),
            state_dir: root.join("state").display().to_string(),
            log_dir: root.join("log").display().to_string(),
        },
        ExecutionSection {
            max_running_actions: 1,
            cancel_grace_ms: 5_000,
            default_stdout_limit_bytes: 1_048_576,
            default_stderr_limit_bytes: 1_048_576,
        },
    )
}

fn rfc3339_before_now(seconds: i64) -> String {
    (OffsetDateTime::now_utc() - TimeDuration::seconds(seconds))
        .format(&Rfc3339)
        .expect("format timestamp")
}

fn rfc3339_after_now(seconds: i64) -> String {
    (OffsetDateTime::now_utc() + TimeDuration::seconds(seconds))
        .format(&Rfc3339)
        .expect("format timestamp")
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

#[cfg(unix)]
#[test]
fn drain_skips_corrupt_queue_head_and_processes_next_item() {
    let root = temp_dir("drain-corrupt-head");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let first = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: sample_plan(),
    })
    .expect("submit first plan");
    let second = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: sample_plan_with_ids("act_002", "req_002"),
    })
    .expect("submit second plan");

    fs::write(
        run_dir
            .join("actions")
            .join(&first.execution_id)
            .join("plan.json"),
        "{bad json",
    )
    .expect("corrupt first plan");

    let outcome = drain_next_with_report(&DrainRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin: test_exec_bin(&root),
        instance_id: "instance-001".to_string(),
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
    })
    .expect("drain queue with corrupt head")
    .expect("drain outcome");

    let queue_state =
        execution_queue::load_or_default(&execution_queue::path_for(&state_dir)).expect("queue");
    let quarantine_path = history::path_for(&state_dir, &first.execution_id);
    let quarantine: history::ExecutionHistoryRecord =
        read_json(&quarantine_path).expect("read quarantine record");

    assert_eq!(outcome.execution_id, second.execution_id);
    assert!(queue_state.items.is_empty());
    assert_eq!(quarantine.state, "quarantined");
    assert!(
        quarantine
            .detail
            .contains("queued execution plan unavailable")
    );
}

#[cfg(unix)]
#[test]
fn drain_quarantines_report_preparation_failure_and_processes_next_item() {
    let root = temp_dir("drain-report-prep-fail");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let first = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: sample_plan(),
    })
    .expect("submit first plan");
    let second = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: sample_plan_with_ids("act_002", "req_002"),
    })
    .expect("submit second plan");

    let first_reporting_path = reporting::path_for(&state_dir, &first.execution_id);
    let blocked_state_tmp_path = first_reporting_path.with_extension("tmp");
    fs::create_dir(&blocked_state_tmp_path).expect("block first reporting state tmp path");

    let outcome = drain_next_with_report(&DrainRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin: test_exec_bin(&root),
        instance_id: "instance-001".to_string(),
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
    })
    .expect("drain queue with report prep failure")
    .expect("drain outcome");

    let queue_state =
        execution_queue::load_or_default(&execution_queue::path_for(&state_dir)).expect("queue");
    let quarantine_path = history::path_for(&state_dir, &first.execution_id);
    let quarantine: history::ExecutionHistoryRecord =
        read_json(&quarantine_path).expect("read quarantine record");
    let second_reporting_path = reporting::path_for(&state_dir, &second.execution_id);
    let first_running_path = running::path_for(&state_dir, &first.execution_id);
    let first_envelope_path =
        reporting_pipeline::envelope_path_for(&state_dir, &first.execution_id);

    assert_eq!(outcome.execution_id, second.execution_id);
    assert!(queue_state.items.is_empty());
    assert!(!first_running_path.exists());
    assert!(!first_reporting_path.exists());
    assert!(!first_envelope_path.exists());
    assert!(second_reporting_path.exists());
    assert_eq!(quarantine.state, "quarantined");
    assert!(
        quarantine
            .detail
            .contains("local execution report preparation failed")
    );
}

#[cfg(unix)]
#[test]
fn daemon_run_once_quarantines_execution_local_failure_without_stopping_loop() {
    let root = temp_dir("run-once-quarantine-exec-failure");
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

    let exec_bin = write_exec_wrapper(
        &root,
        "workdir=\"$3\"\nprintf '{bad json' > \"$workdir/result.json\"\nexit 0",
    );
    let config = standalone_config(&root);
    let snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &config,
        exec_bin: &exec_bin,
    })
    .expect("daemon run once");

    let queue_state =
        execution_queue::load_or_default(&execution_queue::path_for(&state_dir)).expect("queue");
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let quarantine_path = history::path_for(&state_dir, &submitted.execution_id);
    let quarantine: history::ExecutionHistoryRecord =
        read_json(&quarantine_path).expect("read quarantine record");
    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);

    assert_eq!(
        snapshot.state,
        wp_agentd::self_observability::HealthState::Idle
    );
    assert_eq!(snapshot.queue_depth, 0);
    assert_eq!(snapshot.running_count, 0);
    assert_eq!(snapshot.reporting_count, 0);
    assert!(queue_state.items.is_empty());
    assert!(!running_path.exists());
    assert!(!reporting_path.exists());
    assert_eq!(quarantine.state, "quarantined");
    assert!(quarantine.detail.contains("local execution failed"));
}

#[cfg(unix)]
#[test]
fn daemon_run_once_returns_error_when_execution_queue_is_corrupt() {
    let root = temp_dir("run-once-corrupt-queue");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    fs::write(execution_queue::path_for(&state_dir), "{bad json").expect("corrupt queue");

    let config = standalone_config(&root);
    let err = daemon::run_once(&daemon::DaemonLoop {
        config: &config,
        exec_bin: &test_exec_bin(&root),
    })
    .expect_err("corrupt queue should be fatal");

    assert_eq!(err.kind(), std::io::ErrorKind::Other);
}

#[cfg(unix)]
#[test]
fn local_exec_kills_child_when_running_state_persist_fails() {
    let root = temp_dir("running-state-fail");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let running_dir = state_dir.join("running");
    let mut perms = fs::metadata(&running_dir)
        .expect("running metadata")
        .permissions();
    perms.set_mode(0o500);
    fs::set_permissions(&running_dir, perms).expect("set running dir permissions");

    let marker = run_dir
        .join("actions")
        .join("exec_store_fail")
        .join("child_ran");
    let exec_bin = write_exec_wrapper(
        &root,
        "workdir=\"$3\"\nsleep 0.2\necho child > \"$workdir/child_ran\"\nsleep 1",
    );
    let result = execute_local(&LocalExecRequest {
        execution_id: "exec_store_fail".to_string(),
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin,
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
        plan_digest: "digest-store-fail".to_string(),
        request_id: "req_001".to_string(),
        plan: sample_plan(),
    });

    let mut restore = fs::metadata(&running_dir)
        .expect("running metadata after execute")
        .permissions();
    restore.set_mode(0o700);
    fs::set_permissions(&running_dir, restore).expect("restore running dir permissions");

    assert!(result.is_err());
    std::thread::sleep(std::time::Duration::from_millis(400));
    assert!(!marker.exists());
}

#[cfg(unix)]
#[test]
fn daemon_run_once_does_not_rerun_live_execution_from_queue() {
    let root = temp_dir("run-once-live-execution");
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

    let workdir = run_dir.join("actions").join(&submitted.execution_id);
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let running_state = running::RunningExecutionState::new(
        submitted.execution_id.clone(),
        "act_001".to_string(),
        submitted.plan_digest.clone(),
        "req_001".to_string(),
        "running".to_string(),
        workdir.display().to_string(),
        Some(std::process::id()),
        None,
        rfc3339_before_now(1),
        Some(rfc3339_after_now(60)),
        Some("step_collect".to_string()),
        Some(1),
        None,
        None,
        rfc3339_before_now(1),
    );
    write_json_atomic(&running_path, &running_state).expect("write running state");

    let rerun_flag = root.join("rerun.flag");
    let exec_bin = write_exec_wrapper(
        &root,
        &format!("echo rerun > \"{}\"\nexit 88", rerun_flag.display()),
    );
    let config = standalone_config(&root);
    let snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &config,
        exec_bin: &exec_bin,
    })
    .expect("daemon run once");

    let queue_path = execution_queue::path_for(&state_dir);
    let queue_state = execution_queue::load_or_default(&queue_path).expect("queue state");
    assert!(!rerun_flag.exists());
    assert!(running_path.exists());
    assert_eq!(queue_state.items.len(), 1);
    assert_eq!(snapshot.queue_depth, 1);
    assert_eq!(snapshot.running_count, 1);
}

#[cfg(unix)]
#[test]
fn recovery_keeps_expired_execution_blocked_when_process_cannot_be_terminated() {
    let root = temp_dir("recover-expired-live-pid");
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

    let workdir = run_dir.join("actions").join(&submitted.execution_id);
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);
    let running_state = running::RunningExecutionState::new(
        submitted.execution_id.clone(),
        "act_001".to_string(),
        submitted.plan_digest.clone(),
        "req_001".to_string(),
        "running".to_string(),
        workdir.display().to_string(),
        Some(1),
        None,
        rfc3339_before_now(30),
        Some(rfc3339_before_now(5)),
        Some("step_collect".to_string()),
        Some(1),
        None,
        None,
        rfc3339_before_now(5),
    );
    write_json_atomic(&running_path, &running_state).expect("write running state");

    daemon::recover_incomplete_executions(&state_dir, "instance-001").expect("recover");

    let running_state = running::load(&running_path).expect("load running state");
    assert!(running_state.kill_requested_at.is_some());
    assert!(running_path.exists());
    assert!(!reporting_path.exists());
}

#[cfg(unix)]
#[test]
fn daemon_run_once_reports_active_when_execution_is_running() {
    let root = temp_dir("run-once-health-running");
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

    let workdir = run_dir.join("actions").join(&submitted.execution_id);
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let running_state = running::RunningExecutionState::new(
        submitted.execution_id.clone(),
        "act_001".to_string(),
        submitted.plan_digest.clone(),
        "req_001".to_string(),
        "running".to_string(),
        workdir.display().to_string(),
        Some(std::process::id()),
        None,
        rfc3339_before_now(1),
        Some(rfc3339_after_now(60)),
        Some("step_collect".to_string()),
        Some(1),
        None,
        None,
        rfc3339_before_now(1),
    );
    write_json_atomic(&running_path, &running_state).expect("write running state");

    let config = standalone_config(&root);
    let snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &config,
        exec_bin: &test_exec_bin(&root),
    })
    .expect("daemon run once");

    assert_eq!(
        snapshot.state,
        wp_agentd::self_observability::HealthState::Active
    );
    assert_eq!(snapshot.running_count, 1);
    assert_eq!(snapshot.reporting_count, 0);
}

#[cfg(unix)]
#[test]
fn daemon_run_once_reports_active_when_reporting_backlog_exists() {
    let root = temp_dir("run-once-health-reporting");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let reporting_path = reporting::path_for(&state_dir, "exec_reporting");
    reporting::store(
        &reporting_path,
        &reporting::ReportingState::new(
            "exec_reporting".to_string(),
            "act_001".to_string(),
            "digest_reporting".to_string(),
            "req_001".to_string(),
            "failed".to_string(),
            run_dir
                .join("actions")
                .join("exec_reporting")
                .join("result.json")
                .display()
                .to_string(),
            None,
            None,
            None,
            1,
            Some("2026-04-12T10:00:00Z".to_string()),
            Some("cp unavailable".to_string()),
        ),
    )
    .expect("store reporting state");

    let config = standalone_config(&root);
    let snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &config,
        exec_bin: &test_exec_bin(&root),
    })
    .expect("daemon run once");

    assert_eq!(
        snapshot.state,
        wp_agentd::self_observability::HealthState::Active
    );
    assert_eq!(snapshot.queue_depth, 0);
    assert_eq!(snapshot.running_count, 0);
    assert_eq!(snapshot.reporting_count, 1);
}

#[cfg(unix)]
#[test]
fn local_exec_truncates_stdout_and_stderr_to_configured_limits() {
    let root = temp_dir("stream-limits");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let exec_bin = write_exec_wrapper(
        &root,
        "printf '1234567890'\nprintf 'abcdefghij' >&2\nexit 3",
    );
    let outcome = execute_local(&LocalExecRequest {
        execution_id: "exec_stream_limits".to_string(),
        run_dir,
        state_dir,
        exec_bin,
        cancel_grace_ms: 50,
        stdout_limit_bytes: 5,
        stderr_limit_bytes: 4,
        plan_digest: "digest-stream-limits".to_string(),
        request_id: "req_001".to_string(),
        plan: sample_plan(),
    })
    .expect("execute local exec with limits");

    let stdout_log = fs::read_to_string(outcome.workdir.join("stdout.log")).expect("read stdout");
    let stderr_log = fs::read_to_string(outcome.workdir.join("stderr.log")).expect("read stderr");

    assert_eq!(outcome.result.final_status, FinalStatus::Failed);
    assert_eq!(stdout_log, "12345\n[truncated by wp-agentd]\n");
    assert_eq!(stderr_log, "abcd\n[truncated by wp-agentd]\n");
}

#[cfg(unix)]
#[test]
fn local_exec_uses_cancel_grace_before_force_kill() {
    let root = temp_dir("timeout-grace");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let mut plan = sample_plan();
    plan.constraints.max_total_duration_ms = 50;
    let exec_bin = write_exec_wrapper(
        &root,
        "workdir=\"$3\"\ntrap 'echo term > \"$workdir/term.flag\"; exit 0' TERM\nwhile true; do sleep 0.05; done",
    );

    let outcome = execute_local(&LocalExecRequest {
        execution_id: "exec_timeout_grace".to_string(),
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin,
        cancel_grace_ms: 200,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
        plan_digest: "digest-timeout-grace".to_string(),
        request_id: "req_001".to_string(),
        plan,
    })
    .expect("timeout local exec with grace");

    let running_path = running::path_for(&state_dir, "exec_timeout_grace");
    let running_state = running::load(&running_path).expect("load running state");

    assert_eq!(outcome.result.final_status, FinalStatus::TimedOut);
    assert!(outcome.workdir.join("term.flag").exists());
    assert!(running_state.cancel_requested_at.is_some());
    assert!(running_state.kill_requested_at.is_none());
}

#[cfg(unix)]
#[test]
fn local_exec_preserves_exec_result_written_during_timeout_grace() {
    let root = temp_dir("timeout-grace-result");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let mut plan = sample_plan();
    plan.constraints.max_total_duration_ms = 50;
    let exec_bin = write_exec_wrapper(
        &root,
        r#"workdir="$3"
trap 'cat > "$workdir/result.json" <<'"'"'EOF'"'"'
{"api_version":"v1","kind":"action_result","action_id":"act_001","execution_id":"exec_timeout_written_result","final_status":"cancelled","exit_reason":"exec_observed_sigterm","step_records":[{"step_id":"step_collect","attempt":1,"op":"process.list","status":"cancelled","started_at":"2026-04-12T10:00:00Z","finished_at":"2026-04-12T10:00:01Z","duration_ms":1,"error_code":"exec_observed_sigterm","stdout_summary":null,"stderr_summary":null}],"outputs":{"items":[]}}
EOF
exit 0' TERM
while true; do sleep 0.05; done"#,
    );

    let outcome = execute_local(&LocalExecRequest {
        execution_id: "exec_timeout_written_result".to_string(),
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin,
        cancel_grace_ms: 200,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
        plan_digest: "digest-timeout-written-result".to_string(),
        request_id: "req_001".to_string(),
        plan,
    })
    .expect("timeout local exec preserving result");

    let stored_result: serde_json::Value =
        read_json(&outcome.workdir.join("result.json")).expect("read stored result");

    assert_eq!(outcome.result.final_status, FinalStatus::Cancelled);
    assert_eq!(
        outcome.result.exit_reason.as_deref(),
        Some("exec_observed_sigterm")
    );
    assert_eq!(stored_result["final_status"], "cancelled");
}

#[cfg(unix)]
#[test]
fn local_exec_overrides_success_written_during_timeout_grace() {
    let root = temp_dir("timeout-grace-success");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let mut plan = sample_plan();
    plan.constraints.max_total_duration_ms = 50;
    let exec_bin = write_exec_wrapper(
        &root,
        r#"workdir="$3"
trap 'cat > "$workdir/result.json" <<'"'"'EOF'"'"'
{"api_version":"v1","kind":"action_result","action_id":"act_001","execution_id":"exec_timeout_success_result","final_status":"succeeded","exit_reason":null,"step_records":[{"step_id":"step_collect","attempt":1,"op":"process.list","status":"succeeded","started_at":"2026-04-12T10:00:00Z","finished_at":"2026-04-12T10:00:01Z","duration_ms":1,"error_code":null,"stdout_summary":null,"stderr_summary":null}],"outputs":{"items":[]}}
EOF
exit 0' TERM
while true; do sleep 0.05; done"#,
    );

    let outcome = execute_local(&LocalExecRequest {
        execution_id: "exec_timeout_success_result".to_string(),
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin,
        cancel_grace_ms: 200,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
        plan_digest: "digest-timeout-success-result".to_string(),
        request_id: "req_001".to_string(),
        plan,
    })
    .expect("timeout local exec overriding success");

    let stored_result: serde_json::Value =
        read_json(&outcome.workdir.join("result.json")).expect("read stored result");

    assert_eq!(outcome.result.final_status, FinalStatus::TimedOut);
    assert_eq!(
        outcome.result.exit_reason.as_deref(),
        Some("agentd_total_timeout")
    );
    assert_eq!(stored_result["final_status"], "timed_out");
}

#[cfg(target_os = "linux")]
#[test]
fn recovery_does_not_treat_pid_reuse_as_live_when_process_identity_mismatches() {
    let root = temp_dir("recover-mismatched-identity");
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

    let workdir = run_dir.join("actions").join(&submitted.execution_id);
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);
    let running_state = running::RunningExecutionState::new(
        submitted.execution_id.clone(),
        "act_001".to_string(),
        submitted.plan_digest.clone(),
        "req_001".to_string(),
        "running".to_string(),
        workdir.display().to_string(),
        Some(std::process::id()),
        Some("stale-process-identity".to_string()),
        rfc3339_before_now(30),
        Some(rfc3339_before_now(5)),
        Some("step_collect".to_string()),
        Some(1),
        None,
        None,
        rfc3339_before_now(5),
    );
    write_json_atomic(&running_path, &running_state).expect("write running state");

    daemon::recover_incomplete_executions(&state_dir, "instance-001").expect("recover");

    assert!(!running_path.exists());
    assert!(reporting_path.exists());
}

#[cfg(unix)]
#[test]
fn recovery_continues_past_corrupt_running_state() {
    let root = temp_dir("recover-corrupt-running");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    fs::write(state_dir.join("running").join("broken.json"), "{bad json")
        .expect("write broken running state");

    let submitted = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        plan: sample_plan(),
    })
    .expect("submit local plan");
    let workdir = run_dir.join("actions").join(&submitted.execution_id);
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let running_state = running::RunningExecutionState::new(
        submitted.execution_id.clone(),
        "act_001".to_string(),
        submitted.plan_digest.clone(),
        "req_001".to_string(),
        "running".to_string(),
        workdir.display().to_string(),
        Some(4242),
        None,
        "2026-04-12T10:00:00Z".to_string(),
        Some("2026-04-12T10:05:00Z".to_string()),
        Some("step_collect".to_string()),
        Some(1),
        None,
        None,
        "2026-04-12T10:00:01Z".to_string(),
    );
    write_json_atomic(&running_path, &running_state).expect("write valid running state");

    daemon::recover_incomplete_executions(&state_dir, "instance-001").expect("recover");

    let quarantine_path = history::path_for(&state_dir, "broken");
    let quarantine: history::ExecutionHistoryRecord =
        read_json(&quarantine_path).expect("read quarantine record");
    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);

    assert_eq!(quarantine.state, "quarantined");
    assert!(quarantine.detail.contains("running state unreadable"));
    assert!(!state_dir.join("running").join("broken.json").exists());
    assert!(reporting_path.exists());
}

#[cfg(unix)]
#[test]
fn daemon_run_once_quarantines_corrupt_running_execution_without_rerun() {
    let root = temp_dir("run-once-corrupt-running-no-rerun");
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

    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    fs::write(&running_path, "{bad json").expect("write corrupt running state");

    let rerun_flag = root.join("rerun.flag");
    let exec_bin = write_exec_wrapper(
        &root,
        &format!("echo rerun > \"{}\"\nexit 88", rerun_flag.display()),
    );
    let config = standalone_config(&root);
    let snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &config,
        exec_bin: &exec_bin,
    })
    .expect("daemon run once");

    let queue_state =
        execution_queue::load_or_default(&execution_queue::path_for(&state_dir)).expect("queue");
    let quarantine_path = history::path_for(&state_dir, &submitted.execution_id);
    let quarantine: history::ExecutionHistoryRecord =
        read_json(&quarantine_path).expect("read quarantine record");
    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);

    assert!(!rerun_flag.exists());
    assert_eq!(snapshot.queue_depth, 0);
    assert_eq!(snapshot.running_count, 0);
    assert!(queue_state.items.is_empty());
    assert_eq!(quarantine.state, "quarantined");
    assert_eq!(
        quarantine.plan_digest.as_deref(),
        Some(submitted.plan_digest.as_str())
    );
    assert!(quarantine.detail.contains("running state unreadable"));
    assert!(!running_path.exists());
    assert!(!reporting_path.exists());
}

#[cfg(unix)]
#[test]
fn recovery_quarantines_running_execution_when_plan_is_missing() {
    let root = temp_dir("recover-missing-plan");
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

    let workdir = run_dir.join("actions").join(&submitted.execution_id);
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let running_state = running::RunningExecutionState::new(
        submitted.execution_id.clone(),
        "act_001".to_string(),
        submitted.plan_digest.clone(),
        "req_001".to_string(),
        "running".to_string(),
        workdir.display().to_string(),
        Some(4242),
        None,
        "2026-04-12T10:00:00Z".to_string(),
        Some("2026-04-12T10:05:00Z".to_string()),
        Some("step_collect".to_string()),
        Some(1),
        None,
        None,
        "2026-04-12T10:00:01Z".to_string(),
    );
    write_json_atomic(&running_path, &running_state).expect("write running state");
    fs::remove_file(workdir.join("plan.json")).expect("remove plan");

    daemon::recover_incomplete_executions(&state_dir, "instance-001").expect("recover");

    let quarantine_path = history::path_for(&state_dir, &submitted.execution_id);
    let quarantine: history::ExecutionHistoryRecord =
        read_json(&quarantine_path).expect("read quarantine record");
    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);

    assert_eq!(quarantine.state, "quarantined");
    assert_eq!(
        quarantine.plan_digest.as_deref(),
        Some(submitted.plan_digest.as_str())
    );
    assert!(
        quarantine
            .detail
            .contains("running execution plan unavailable")
    );
    assert!(!running_path.exists());
    assert!(!reporting_path.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn daemon_run_once_does_not_block_on_mismatched_process_identity() {
    let root = temp_dir("run-once-mismatched-identity");
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

    let workdir = run_dir.join("actions").join(&submitted.execution_id);
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let running_state = running::RunningExecutionState::new(
        submitted.execution_id.clone(),
        "act_001".to_string(),
        submitted.plan_digest.clone(),
        "req_001".to_string(),
        "running".to_string(),
        workdir.display().to_string(),
        Some(std::process::id()),
        Some("stale-process-identity".to_string()),
        rfc3339_before_now(1),
        Some(rfc3339_after_now(60)),
        Some("step_collect".to_string()),
        Some(1),
        None,
        None,
        rfc3339_before_now(1),
    );
    write_json_atomic(&running_path, &running_state).expect("write running state");

    let config = standalone_config(&root);
    let snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &config,
        exec_bin: &test_exec_bin(&root),
    })
    .expect("daemon run once");

    let queue_state =
        execution_queue::load_or_default(&execution_queue::path_for(&state_dir)).expect("queue");
    let reporting_path = reporting::path_for(&state_dir, &submitted.execution_id);

    assert_eq!(snapshot.queue_depth, 0);
    assert_eq!(snapshot.running_count, 0);
    assert!(queue_state.items.is_empty());
    assert!(!running_path.exists());
    assert!(reporting_path.exists());
}

#[cfg(unix)]
#[test]
fn drain_does_not_rerun_expired_execution_when_process_cannot_be_terminated() {
    let root = temp_dir("drain-recover-expired");
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

    let workdir = run_dir.join("actions").join(&submitted.execution_id);
    let running_path = running::path_for(&state_dir, &submitted.execution_id);
    let running_state = running::RunningExecutionState::new(
        submitted.execution_id.clone(),
        "act_001".to_string(),
        submitted.plan_digest.clone(),
        "req_001".to_string(),
        "running".to_string(),
        workdir.display().to_string(),
        Some(1),
        None,
        rfc3339_before_now(30),
        Some(rfc3339_before_now(5)),
        Some("step_collect".to_string()),
        Some(1),
        None,
        None,
        rfc3339_before_now(5),
    );
    write_json_atomic(&running_path, &running_state).expect("write running state");

    let rerun_flag = root.join("rerun.flag");
    let exec_bin = write_exec_wrapper(
        &root,
        &format!("echo rerun > \"{}\"\nexit 88", rerun_flag.display()),
    );

    let outcome = drain_next_with_report(&DrainRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin,
        instance_id: "instance-001".to_string(),
        cancel_grace_ms: 5_000,
        stdout_limit_bytes: 1_048_576,
        stderr_limit_bytes: 1_048_576,
    })
    .expect("drain expired running execution");

    let queue_path = execution_queue::path_for(&state_dir);
    let queue_state = execution_queue::load_or_default(&queue_path).expect("queue state");
    let running_state = running::load(&running_path).expect("load running state");
    assert!(outcome.is_none());
    assert!(!rerun_flag.exists());
    assert!(running_state.kill_requested_at.is_some());
    assert!(running_path.exists());
    assert_eq!(queue_state.items.len(), 1);
}
