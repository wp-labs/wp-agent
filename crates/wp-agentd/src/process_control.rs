//! Shared process lifecycle helpers for runtime execution control.

use std::io;
use std::path::Path;

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use wp_agent_shared::time::now_rfc3339;

use crate::state_store::running;

#[path = "process_control_support.rs"]
mod support;

use support::{ProcessIdentityState, derive_running_state_status, process_identity_state};
pub(crate) use support::{force_kill, process_identity, send_terminate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalRequestKind {
    Cancel,
    Kill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunningStateStatus {
    Inactive,
    Active,
    Expired,
}

pub(crate) fn inspect_running_state(
    state: &running::RunningExecutionState,
) -> io::Result<RunningStateStatus> {
    let Some(pid) = state.pid else {
        return Ok(RunningStateStatus::Inactive);
    };
    Ok(derive_running_state_status(
        process_identity_state(pid, state.process_identity.as_deref())?,
        deadline_has_elapsed(state.deadline_at.as_deref()),
    ))
}

pub(crate) fn handle_expired_running_state(
    state: &mut running::RunningExecutionState,
    running_path: &Path,
) -> io::Result<bool> {
    force_stop_expired_process(state, running_path)
}

pub(crate) fn record_signal_request(
    running_path: &Path,
    kind: SignalRequestKind,
) -> io::Result<()> {
    if !running_path.exists() {
        return Ok(());
    }

    let mut state = running::load(running_path)?;
    let requested_at = now_rfc3339();
    match kind {
        SignalRequestKind::Cancel => {
            if state.cancel_requested_at.is_none() {
                state.cancel_requested_at = Some(requested_at.clone());
            }
            state.state = "cancelling".to_string();
        }
        SignalRequestKind::Kill => {
            state.kill_requested_at = Some(requested_at.clone());
            state.state = "kill_requested".to_string();
        }
    }
    state.updated_at = requested_at;
    running::store(running_path, &state)
}

pub(crate) fn deadline_has_elapsed(deadline_at: Option<&str>) -> bool {
    let Some(deadline_at) = deadline_at else {
        return false;
    };
    let Ok(deadline) = OffsetDateTime::parse(deadline_at, &Rfc3339) else {
        return false;
    };
    deadline <= OffsetDateTime::now_utc()
}

fn force_stop_expired_process(
    state: &mut running::RunningExecutionState,
    running_path: &Path,
) -> io::Result<bool> {
    let Some(pid) = state.pid else {
        return Ok(false);
    };
    match process_identity_state(pid, state.process_identity.as_deref())? {
        ProcessIdentityState::MissingProcess | ProcessIdentityState::Mismatch => return Ok(false),
        // Identity became unreadable after spawn. Stay blocked rather than recovering or
        // signaling a process we can no longer verify belongs to this execution.
        ProcessIdentityState::Unavailable => return Ok(true),
        ProcessIdentityState::Matches => {}
    }

    record_signal_request(running_path, SignalRequestKind::Kill)?;
    state.kill_requested_at = running::load(running_path)?.kill_requested_at;
    force_kill(pid)
}

#[cfg(test)]
#[path = "process_control_tests.rs"]
mod tests;
