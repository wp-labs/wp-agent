use std::fs;

use warp_insight_shared::fs::read_json;
use warp_insightd::bootstrap;
use warp_insightd::reporting_pipeline;
use warp_insightd::scheduler::{
    DrainRequest, SchedulerRequest, drain_next_with_report, submit_local_plan,
};
use warp_insightd::state_store::{execution_queue, history, reporting, running};

use super::super::common::{sample_plan, sample_plan_with_ids, temp_dir, test_exec_bin};

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
