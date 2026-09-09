#[cfg(target_os = "linux")]
use super::support::parse_linux_proc_state;
#[cfg(all(unix, not(target_os = "linux")))]
use super::support::process_is_zombie_via_ps;
use super::support::{
    classify_process_identity, derive_running_state_status, ProcessIdentityState,
};
#[cfg(unix)]
use super::support::{process_exists, process_identity_state};
use super::{inspect_running_state, RunningStateStatus};
use crate::state_store::running::RunningExecutionState;

#[test]
fn classify_process_identity_marks_unavailable_as_distinct_from_mismatch() {
    assert_eq!(
        classify_process_identity(Some("expected-token"), None),
        ProcessIdentityState::Unavailable
    );
    assert_eq!(
        classify_process_identity(Some("expected-token"), Some("other-token")),
        ProcessIdentityState::Mismatch
    );
}

#[test]
fn classify_process_identity_treats_missing_expectation_as_match() {
    assert_eq!(
        classify_process_identity(None, None),
        ProcessIdentityState::Matches
    );
}

#[cfg(unix)]
#[test]
fn process_identity_state_reports_missing_for_pid_zero() {
    assert_eq!(
        process_identity_state(0, None).expect("pid zero short-circuits before syscalls"),
        ProcessIdentityState::MissingProcess
    );
}

#[cfg(unix)]
#[test]
fn process_identity_state_matches_live_process_without_identity_expectation() {
    assert_eq!(
        process_identity_state(std::process::id(), None).expect("probe current process"),
        ProcessIdentityState::Matches
    );
}

#[cfg(unix)]
#[test]
fn process_exists_probes_current_process_and_zero() {
    assert!(process_exists(std::process::id()).expect("current process exists"));
    assert!(!process_exists(0).expect("pid zero never exists"));
}

#[test]
fn inspect_running_state_reports_expired_deadline_without_side_effects() {
    assert_eq!(
        derive_running_state_status(ProcessIdentityState::Matches, true),
        RunningStateStatus::Expired
    );
    assert_eq!(
        derive_running_state_status(ProcessIdentityState::Matches, false),
        RunningStateStatus::Active
    );
    assert_eq!(
        derive_running_state_status(ProcessIdentityState::Unavailable, true),
        RunningStateStatus::Active
    );
    assert_eq!(
        derive_running_state_status(ProcessIdentityState::Mismatch, true),
        RunningStateStatus::Inactive
    );
}

#[test]
fn inspect_running_state_without_pid_is_inactive() {
    let state = RunningExecutionState::builder(
        "exec_001".to_string(),
        "act_001".to_string(),
        "running".to_string(),
        "/tmp/workdir".to_string(),
    )
    .plan_digest("digest_001".to_string())
    .request_id("req_001".to_string())
    .started_at("2026-04-12T10:00:00Z".to_string())
    .updated_at("2026-04-12T10:00:00Z".to_string())
    .pid(None)
    .process_identity(None)
    .deadline_at(Some("2000-01-01T00:00:00Z".to_string()))
    .current_step_id(None)
    .attempt(Some(1))
    .cancel_requested_at(None)
    .kill_requested_at(None)
    .build();

    assert_eq!(
        inspect_running_state(&state).expect("inspect state"),
        RunningStateStatus::Inactive
    );
}

#[cfg(target_os = "linux")]
#[test]
fn parse_linux_proc_state_detects_zombie_state() {
    assert_eq!(
        parse_linux_proc_state("1234 (wist-exec) Z 1 2 3 4 5"),
        Some('Z')
    );
}

#[cfg(target_os = "linux")]
#[test]
fn parse_linux_proc_state_returns_none_for_invalid_input() {
    assert_eq!(parse_linux_proc_state("not-a-proc-stat-line"), None);
}

#[cfg(all(unix, not(target_os = "linux")))]
#[test]
fn process_is_zombie_via_ps_treats_missing_command_as_not_zombie() {
    let missing = "warp-agentd-test-missing-ps-command";
    assert!(!process_is_zombie_via_ps(std::process::id(), missing)
        .expect("missing ps should be treated as a benign fallback"));
}

#[cfg(all(unix, not(target_os = "linux")))]
#[test]
fn process_is_zombie_via_ps_sees_current_process_as_not_zombie() {
    assert!(!process_is_zombie_via_ps(std::process::id(), "ps")
        .expect("ps probe of the current process succeeds"));
}
