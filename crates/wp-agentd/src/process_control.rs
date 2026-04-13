//! Shared process lifecycle helpers for runtime execution control.

use std::io;
use std::path::Path;
#[cfg(unix)]
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use wp_agent_shared::time::now_rfc3339;

use crate::state_store::running;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalRequestKind {
    Cancel,
    Kill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessIdentityState {
    MissingProcess,
    Matches,
    Mismatch,
    Unavailable,
}

pub(crate) fn running_state_is_active(
    state: &mut running::RunningExecutionState,
    running_path: &Path,
) -> io::Result<bool> {
    let Some(pid) = state.pid else {
        return Ok(false);
    };
    match process_identity_state(pid, state.process_identity.as_deref())? {
        ProcessIdentityState::MissingProcess | ProcessIdentityState::Mismatch => Ok(false),
        ProcessIdentityState::Unavailable => Ok(true),
        ProcessIdentityState::Matches => {
            if deadline_has_elapsed(state.deadline_at.as_deref()) {
                force_stop_expired_process(state, running_path)
            } else {
                Ok(true)
            }
        }
    }
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

pub(crate) fn process_identity(pid: u32) -> io::Result<Option<String>> {
    if pid == 0 {
        return Ok(None);
    }
    process_identity_token(pid)
}

#[cfg(unix)]
pub(crate) fn send_terminate(pid: u32) -> io::Result<()> {
    send_signal(pid, libc::SIGTERM)
}

#[cfg(not(unix))]
pub(crate) fn send_terminate(_pid: u32) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
pub(crate) fn force_kill(pid: u32) -> io::Result<bool> {
    if pid == 0 {
        return Ok(false);
    }

    match send_signal(pid, libc::SIGKILL) {
        Ok(()) => wait_for_exit(pid, Duration::from_millis(250)),
        Err(err) if err.raw_os_error() == Some(libc::EPERM) => Ok(true),
        Err(err) if err.raw_os_error() == Some(libc::ESRCH) => Ok(false),
        Err(err) => Err(err),
    }
}

#[cfg(not(unix))]
pub(crate) fn force_kill(_pid: u32) -> io::Result<bool> {
    Ok(false)
}

#[cfg(unix)]
pub(crate) fn process_exists(pid: u32) -> io::Result<bool> {
    if pid == 0 {
        return Ok(false);
    }

    let rc = unsafe { libc::kill(pid as i32, 0) };
    if rc == 0 {
        return Ok(true);
    }

    let err = io::Error::last_os_error();
    match err.raw_os_error() {
        Some(code) if code == libc::ESRCH => Ok(false),
        Some(code) if code == libc::EPERM => Ok(true),
        _ => Err(err),
    }
}

#[cfg(not(unix))]
pub(crate) fn process_exists(_pid: u32) -> io::Result<bool> {
    Ok(false)
}

fn process_identity_state(pid: u32, expected: Option<&str>) -> io::Result<ProcessIdentityState> {
    if !process_exists(pid)? {
        return Ok(ProcessIdentityState::MissingProcess);
    }

    let actual = match expected {
        Some(_) => process_identity(pid)?,
        None => None,
    };
    Ok(classify_process_identity(expected, actual.as_deref()))
}

fn classify_process_identity(expected: Option<&str>, actual: Option<&str>) -> ProcessIdentityState {
    match expected {
        None => ProcessIdentityState::Matches,
        Some(expected) => match actual {
            Some(actual) if actual == expected => ProcessIdentityState::Matches,
            Some(_) => ProcessIdentityState::Mismatch,
            None => ProcessIdentityState::Unavailable,
        },
    }
}

#[cfg(unix)]
fn send_signal(pid: u32, signal: i32) -> io::Result<()> {
    if pid == 0 {
        return Ok(());
    }

    let rc = unsafe { libc::kill(pid as i32, signal) };
    if rc == 0 {
        return Ok(());
    }

    Err(io::Error::last_os_error())
}

#[cfg(unix)]
fn wait_for_exit(pid: u32, timeout: Duration) -> io::Result<bool> {
    let deadline = Instant::now() + timeout;
    loop {
        if !process_exists(pid)? {
            return Ok(false);
        }
        if process_is_zombie(pid)? {
            return Ok(false);
        }
        if Instant::now() >= deadline {
            return Ok(true);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(target_os = "linux")]
fn process_identity_token(pid: u32) -> io::Result<Option<String>> {
    let stat_path = format!("/proc/{pid}/stat");
    let stat = match std::fs::read_to_string(stat_path) {
        Ok(stat) => stat,
        Err(err)
            if matches!(
                err.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
            ) =>
        {
            return Ok(None);
        }
        Err(err) => return Err(err),
    };
    let Some((_, tail)) = stat.rsplit_once(") ") else {
        return Ok(None);
    };
    let fields: Vec<&str> = tail.split_whitespace().collect();
    if fields.len() <= 19 {
        return Ok(None);
    }
    Ok(Some(format!("linux_proc_start:{}", fields[19])))
}

#[cfg(all(unix, not(target_os = "linux")))]
fn process_identity_token(pid: u32) -> io::Result<Option<String>> {
    let output = match Command::new("ps")
        .args(["-o", "lstart=", "-p", &pid.to_string()])
        .output()
    {
        Ok(output) => output,
        Err(err) if err.kind() == io::ErrorKind::PermissionDenied => return Ok(None),
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err),
    };
    if !output.status.success() {
        return Ok(None);
    }
    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!("ps_lstart:{token}")))
}

#[cfg(not(unix))]
fn process_identity_token(_pid: u32) -> io::Result<Option<String>> {
    Ok(None)
}

#[cfg(target_os = "linux")]
fn process_is_zombie(pid: u32) -> io::Result<bool> {
    let stat_path = format!("/proc/{pid}/stat");
    let stat = match std::fs::read_to_string(stat_path) {
        Ok(stat) => stat,
        Err(err)
            if matches!(
                err.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
            ) =>
        {
            return Ok(false);
        }
        Err(err) => return Err(err),
    };
    Ok(parse_linux_proc_state(&stat) == Some('Z'))
}

#[cfg(target_os = "linux")]
fn parse_linux_proc_state(stat: &str) -> Option<char> {
    let (_, tail) = stat.rsplit_once(") ")?;
    tail.chars().next()
}

#[cfg(all(unix, not(target_os = "linux")))]
fn process_is_zombie(pid: u32) -> io::Result<bool> {
    process_is_zombie_via_ps(pid, "ps")
}

#[cfg(all(unix, not(target_os = "linux")))]
fn process_is_zombie_via_ps(pid: u32, program: &str) -> io::Result<bool> {
    let output = match Command::new(program)
        .args(["-o", "stat=", "-p", &pid.to_string()])
        .output()
    {
        Ok(output) => output,
        Err(err) if err.kind() == io::ErrorKind::PermissionDenied => return Ok(false),
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err),
    };
    if !output.status.success() {
        return Ok(false);
    }
    let stat = String::from_utf8_lossy(&output.stdout);
    Ok(stat.trim_start().starts_with('Z'))
}

#[cfg(not(unix))]
fn process_is_zombie(_pid: u32) -> io::Result<bool> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    use super::parse_linux_proc_state;
    #[cfg(all(unix, not(target_os = "linux")))]
    use super::process_is_zombie_via_ps;
    use super::{ProcessIdentityState, classify_process_identity};

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

    #[cfg(target_os = "linux")]
    #[test]
    fn parse_linux_proc_state_detects_zombie_state() {
        assert_eq!(
            parse_linux_proc_state("1234 (wp-agent-exec) Z 1 2 3 4 5"),
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
        let missing = "wp-agentd-test-missing-ps-command";
        assert!(
            !process_is_zombie_via_ps(std::process::id(), missing)
                .expect("missing ps should be treated as a benign fallback")
        );
    }
}
