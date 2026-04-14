use std::fs;

use wp_agent_contracts::action_result::FinalStatus;
use wp_agent_shared::fs::read_json;
use wp_agentd::bootstrap;
use wp_agentd::local_exec::{LocalExecRequest, execute as execute_local};
use wp_agentd::state_store::running;

use super::common::{sample_plan, temp_dir, write_exec_wrapper};

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
    use std::os::unix::fs::PermissionsExt;
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
