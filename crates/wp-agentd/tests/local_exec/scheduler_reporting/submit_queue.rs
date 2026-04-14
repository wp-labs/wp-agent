use std::fs;

use wp_agentd::bootstrap;
use wp_agentd::scheduler::{
    DrainRequest, SchedulerRequest, drain_next_with_report, submit_local_plan,
};
use wp_agentd::state_store::{execution_queue, reporting};

use super::super::common::{sample_plan, temp_dir, test_exec_bin};

#[cfg(unix)]
#[test]
fn submit_local_plan_does_not_enqueue_when_queue_store_fails() {
    let root = temp_dir("submit-queue-fail");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    use std::os::unix::fs::PermissionsExt;
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
