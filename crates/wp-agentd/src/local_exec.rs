//! Local execution controller for `wp-agent-exec`.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use wp_agent_contracts::action_plan::ActionPlanContract;
use wp_agent_contracts::action_result::{
    ActionOutputs, ActionResultContract, FinalStatus, StepActionRecord, StepStatus,
};
use wp_agent_shared::fs::{read_json, write_json_atomic};
use wp_agent_shared::paths::{
    ACTIONS_DIR, WORKDIR_PLAN_FILE, WORKDIR_RESULT_FILE, WORKDIR_RUNTIME_FILE, WORKDIR_STATE_FILE,
};
use wp_agent_shared::time::{after_millis_rfc3339, now_rfc3339};

use crate::execution_support::final_state_name;
use crate::process_control::{
    SignalRequestKind, process_identity, record_signal_request, send_terminate,
};
use crate::state_store::running;

#[derive(Debug, Clone)]
pub struct LocalExecRequest {
    pub execution_id: String,
    pub run_dir: PathBuf,
    pub state_dir: PathBuf,
    pub exec_bin: PathBuf,
    pub cancel_grace_ms: u64,
    pub stdout_limit_bytes: u64,
    pub stderr_limit_bytes: u64,
    pub plan_digest: String,
    pub request_id: String,
    pub plan: ActionPlanContract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExitClassification {
    Completed(ExitStatus),
    CompletedAfterTimeout(ExitStatus),
    TimedOut,
}

#[derive(Debug, Clone)]
pub struct LocalExecOutcome {
    pub execution_id: String,
    pub workdir: PathBuf,
    pub result: ActionResultContract,
}

const STREAM_TRUNCATED_MARKER: &str = "\n[truncated by wp-agentd]\n";

pub fn execute(request: &LocalExecRequest) -> io::Result<LocalExecOutcome> {
    let workdir = request
        .run_dir
        .join(ACTIONS_DIR)
        .join(&request.execution_id);
    fs::create_dir_all(&workdir)?;

    let runtime = ExecRuntimeContext {
        execution_id: request.execution_id.clone(),
        spawned_at: now_rfc3339(),
        deadline_at: Some(after_millis_rfc3339(
            request.plan.constraints.max_total_duration_ms,
        )),
        agent_id: request.plan.target.agent_id.clone(),
        node_id: request.plan.target.node_id.clone(),
        workdir: workdir.display().to_string(),
    };

    write_json_atomic(&workdir.join(WORKDIR_PLAN_FILE), &request.plan)?;
    write_json_atomic(&workdir.join(WORKDIR_RUNTIME_FILE), &runtime)?;

    let stdout_log_path = workdir.join("stdout.log");
    let stderr_log_path = workdir.join("stderr.log");
    let stdout_log = File::create(&stdout_log_path)?;
    let stderr_log = File::create(&stderr_log_path)?;

    let mut child = Command::new(&request.exec_bin)
        .arg("run")
        .arg("--workdir")
        .arg(&workdir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout_reader = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("child stdout pipe was not available"))?;
    let stderr_reader = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("child stderr pipe was not available"))?;
    let stdout_capture =
        spawn_stream_capture(stdout_reader, stdout_log, request.stdout_limit_bytes);
    let stderr_capture =
        spawn_stream_capture(stderr_reader, stderr_log, request.stderr_limit_bytes);

    let started_at = now_rfc3339();
    let running_path = running::path_for(&request.state_dir, &request.execution_id);
    let running_state = running::RunningExecutionState::new(
        request.execution_id.clone(),
        request.plan.meta.action_id.clone(),
        request.plan_digest.clone(),
        request.request_id.clone(),
        "spawned".to_string(),
        workdir.display().to_string(),
        child.id().into(),
        process_identity(child.id())?,
        started_at.clone(),
        runtime.deadline_at.clone(),
        None,
        Some(1),
        None,
        None,
        started_at,
    );
    if let Err(err) = running::store(&running_path, &running_state) {
        terminate_child(&mut child)?;
        join_capture(stdout_capture, "stdout")?;
        join_capture(stderr_capture, "stderr")?;
        return Err(io::Error::new(
            err.kind(),
            format!(
                "failed to persist running state for {} after spawn: {err}",
                request.execution_id
            ),
        ));
    }

    let exit_status = wait_for_child(
        &mut child,
        request.plan.constraints.max_total_duration_ms,
        request.cancel_grace_ms,
        &running_path,
    )?;
    join_capture(stdout_capture, "stdout")?;
    join_capture(stderr_capture, "stderr")?;
    let result_path = workdir.join(WORKDIR_RESULT_FILE);
    let result = match exit_status {
        ExitClassification::Completed(status) if result_path.exists() => {
            let result: ActionResultContract = read_json(&result_path)?;
            if !status.success() && result.final_status == FinalStatus::Succeeded {
                return Err(io::Error::other(
                    "exec process exited non-zero with succeeded result",
                ));
            }
            result
        }
        ExitClassification::CompletedAfterTimeout(_status) if result_path.exists() => {
            let result: ActionResultContract = read_json(&result_path)?;
            if result.final_status == FinalStatus::Succeeded {
                write_timed_out_result(request, &workdir, &result_path)?
            } else {
                result
            }
        }
        ExitClassification::TimedOut | ExitClassification::CompletedAfterTimeout(_) => {
            write_timed_out_result(request, &workdir, &result_path)?
        }
        ExitClassification::Completed(status) => {
            let reason = match status.code() {
                Some(code) => format!("exec_exit_{code}"),
                None => "exec_terminated_by_signal".to_string(),
            };
            let result = synthesize_result(request, FinalStatus::Failed, &reason, "failed");
            write_json_atomic(&result_path, &result)?;
            write_exec_state(
                &workdir,
                &request.execution_id,
                &request.plan.meta.action_id,
                "failed",
                Some(reason),
                "agentd synthesized failure result after abnormal exec exit",
            )?;
            result
        }
    };

    let signal_state = running::load(&running_path).ok();
    let finished_state = running::RunningExecutionState::new(
        request.execution_id.clone(),
        request.plan.meta.action_id.clone(),
        request.plan_digest.clone(),
        request.request_id.clone(),
        final_state_name(&result).to_string(),
        workdir.display().to_string(),
        child.id().into(),
        running_state.process_identity.clone(),
        running_state.started_at,
        runtime.deadline_at,
        None,
        Some(1),
        signal_state
            .as_ref()
            .and_then(|state| state.cancel_requested_at.clone()),
        signal_state
            .as_ref()
            .and_then(|state| state.kill_requested_at.clone()),
        now_rfc3339(),
    );
    running::store(&running_path, &finished_state)?;

    Ok(LocalExecOutcome {
        execution_id: request.execution_id.clone(),
        workdir,
        result,
    })
}

fn write_timed_out_result(
    request: &LocalExecRequest,
    workdir: &std::path::Path,
    result_path: &std::path::Path,
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

fn wait_for_child(
    child: &mut std::process::Child,
    timeout_ms: u64,
    cancel_grace_ms: u64,
    running_path: &std::path::Path,
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
                if let Some(_status) = child.try_wait()? {
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

fn spawn_stream_capture<R>(
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

fn join_capture(handle: JoinHandle<io::Result<()>>, stream_name: &str) -> io::Result<()> {
    match handle.join() {
        Ok(result) => result,
        Err(_) => Err(io::Error::other(format!(
            "{stream_name} capture thread panicked"
        ))),
    }
}

fn terminate_child(child: &mut Child) -> io::Result<()> {
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

fn synthesize_result(
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
            finished_at: Some(finished_at),
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

fn write_exec_state(
    workdir: &std::path::Path,
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

pub fn next_execution_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_nanos();
    format!("exec_{ts}")
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecRuntimeContext {
    pub execution_id: String,
    pub spawned_at: String,
    pub deadline_at: Option<String>,
    pub agent_id: String,
    pub node_id: String,
    pub workdir: String,
}

#[cfg(test)]
mod tests {
    use super::next_execution_id;

    #[test]
    fn execution_id_has_prefix() {
        assert!(next_execution_id().starts_with("exec_"));
    }
}
