use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Child, ExitStatus};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use wp_agent_contracts::action_result::{
    ActionOutputs, ActionResultContract, FinalStatus, StepActionRecord, StepStatus,
};
use wp_agent_shared::fs::write_json_atomic;
use wp_agent_shared::paths::WORKDIR_STATE_FILE;
use wp_agent_shared::time::now_rfc3339;

use crate::local_exec::LocalExecRequest;
use crate::process_control::{SignalRequestKind, record_signal_request, send_terminate};

const STREAM_TRUNCATED_MARKER: &str = "\n[truncated by wp-agentd]\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExitClassification {
    Completed(ExitStatus),
    CompletedAfterTimeout(ExitStatus),
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExecRuntimeContext {
    pub execution_id: String,
    pub spawned_at: String,
    pub deadline_at: Option<String>,
    pub agent_id: String,
    pub node_id: String,
    pub workdir: String,
}

pub(super) fn write_timed_out_result(
    request: &LocalExecRequest,
    workdir: &Path,
    result_path: &Path,
) -> io::Result<ActionResultContract> {
    let result = synthesize_result(
        request,
        FinalStatus::TimedOut,
        "agentd_total_timeout",
        "timed_out",
    );
    write_json_atomic(result_path, &result)?;
    write_exec_state(
        workdir,
        &request.execution_id,
        &request.plan.meta.action_id,
        "timed_out",
        Some("agentd_total_timeout".to_string()),
        "agentd timed out execution and synthesized final result",
    )?;
    Ok(result)
}

pub(super) fn wait_for_child(
    child: &mut Child,
    timeout_ms: u64,
    cancel_grace_ms: u64,
    running_path: &Path,
) -> io::Result<ExitClassification> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(1));
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(ExitClassification::Completed(status));
        }
        if Instant::now() >= deadline {
            if let Some(status) = child.try_wait()? {
                return Ok(ExitClassification::Completed(status));
            }
            let pid = child.id();
            record_signal_request(running_path, SignalRequestKind::Cancel)?;
            if let Err(err) = send_terminate(pid) {
                if let Some(status) = child.try_wait()? {
                    return Ok(ExitClassification::CompletedAfterTimeout(status));
                }
                if err.kind() != io::ErrorKind::InvalidInput {
                    return Err(err);
                }
            }
            let cancel_deadline = Instant::now() + Duration::from_millis(cancel_grace_ms.max(1));
            while Instant::now() < cancel_deadline {
                if let Some(status) = child.try_wait()? {
                    return Ok(ExitClassification::CompletedAfterTimeout(status));
                }
                thread::sleep(Duration::from_millis(10));
            }
            record_signal_request(running_path, SignalRequestKind::Kill)?;
            if let Err(err) = child.kill() {
                if child.try_wait()?.is_some() {
                    return Ok(ExitClassification::TimedOut);
                }
                if err.kind() != io::ErrorKind::InvalidInput {
                    return Err(err);
                }
            }
            let _ = child.wait()?;
            return Ok(ExitClassification::TimedOut);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub(super) fn spawn_stream_capture<R>(
    mut reader: R,
    mut output: File,
    limit_bytes: u64,
) -> JoinHandle<io::Result<()>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut written = 0u64;
        let mut truncated = false;

        loop {
            let read = reader.read(&mut buf)?;
            if read == 0 {
                break;
            }

            if written < limit_bytes {
                let remaining = (limit_bytes - written) as usize;
                let to_write = read.min(remaining);
                if to_write > 0 {
                    output.write_all(&buf[..to_write])?;
                    written += to_write as u64;
                }
                if to_write < read && !truncated {
                    output.write_all(STREAM_TRUNCATED_MARKER.as_bytes())?;
                    truncated = true;
                }
            }
        }

        output.sync_all()?;
        Ok(())
    })
}

pub(super) fn join_capture(
    handle: JoinHandle<io::Result<()>>,
    stream_name: &str,
) -> io::Result<()> {
    match handle.join() {
        Ok(result) => result,
        Err(_) => Err(io::Error::other(format!(
            "{stream_name} capture thread panicked"
        ))),
    }
}

pub(super) fn terminate_child(child: &mut Child) -> io::Result<()> {
    if child.try_wait()?.is_some() {
        return Ok(());
    }

    match child.kill() {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::InvalidInput => {}
        Err(err) => return Err(err),
    }
    let _ = child.wait()?;
    Ok(())
}

pub(super) fn synthesize_result(
    request: &LocalExecRequest,
    final_status: FinalStatus,
    error_code: &str,
    step_status: &str,
) -> ActionResultContract {
    let finished_at = now_rfc3339();
    let step_status = match step_status {
        "timed_out" => StepStatus::TimedOut,
        "cancelled" => StepStatus::Cancelled,
        _ => StepStatus::Failed,
    };
    ActionResultContract {
        request_id: Some(request.request_id.clone()),
        exit_reason: Some(error_code.to_string()),
        step_records: vec![StepActionRecord {
            step_id: request.plan.program.entry.clone(),
            attempt: 1,
            op: request
                .plan
                .program
                .steps
                .iter()
                .find(|step| step.id == request.plan.program.entry)
                .and_then(|step| step.op.clone()),
            status: step_status,
            started_at: finished_at.clone(),
            finished_at: Some(finished_at.clone()),
            duration_ms: None,
            error_code: Some(error_code.to_string()),
            stdout_summary: None,
            stderr_summary: None,
            resource_usage: None,
        }],
        outputs: ActionOutputs::default(),
        started_at: Some(finished_at.clone()),
        finished_at: Some(finished_at),
        ..ActionResultContract::new(
            request.plan.meta.action_id.clone(),
            request.execution_id.clone(),
            final_status,
        )
    }
}

pub(super) fn write_exec_state(
    workdir: &Path,
    execution_id: &str,
    action_id: &str,
    state: &str,
    reason_code: Option<String>,
    detail: &str,
) -> io::Result<()> {
    let state_path = workdir.join(WORKDIR_STATE_FILE);
    let value = serde_json::json!({
        "execution_id": execution_id,
        "action_id": action_id,
        "state": state,
        "updated_at": now_rfc3339(),
        "step_id": serde_json::Value::Null,
        "attempt": serde_json::Value::Null,
        "reason_code": reason_code,
        "detail": detail,
    });
    write_json_atomic(&state_path, &value)
}
