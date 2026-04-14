//! Local execution controller for `wp-agent-exec`.

use std::fs::{self, File};
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use wp_agent_contracts::action_plan::ActionPlanContract;
use wp_agent_contracts::action_result::{ActionResultContract, FinalStatus};
use wp_agent_shared::fs::{read_json, write_json_atomic};
use wp_agent_shared::paths::{
    ACTIONS_DIR, WORKDIR_PLAN_FILE, WORKDIR_RESULT_FILE, WORKDIR_RUNTIME_FILE,
};
use wp_agent_shared::time::{after_millis_rfc3339, now_rfc3339};

use crate::execution_support::final_state_name;
use crate::process_control::process_identity;
use crate::state_store::running;

#[path = "local_exec_support.rs"]
mod support;

use support::{
    ExecRuntimeContext, ExitClassification, join_capture, spawn_stream_capture, synthesize_result,
    terminate_child, wait_for_child, write_exec_state, write_timed_out_result,
};

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

#[derive(Debug, Clone)]
pub struct LocalExecOutcome {
    pub execution_id: String,
    pub workdir: PathBuf,
    pub result: ActionResultContract,
}

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

    let stdout_log = File::create(workdir.join("stdout.log"))?;
    let stderr_log = File::create(workdir.join("stderr.log"))?;

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
    let result = load_or_synthesize_result(request, &workdir, &result_path, exit_status)?;

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

fn load_or_synthesize_result(
    request: &LocalExecRequest,
    workdir: &std::path::Path,
    result_path: &std::path::Path,
    exit_status: ExitClassification,
) -> io::Result<ActionResultContract> {
    match exit_status {
        ExitClassification::Completed(status) if result_path.exists() => {
            let result: ActionResultContract = read_json(result_path)?;
            if !status.success() && result.final_status == FinalStatus::Succeeded {
                return Err(io::Error::other(
                    "exec process exited non-zero with succeeded result",
                ));
            }
            Ok(result)
        }
        ExitClassification::CompletedAfterTimeout(_) if result_path.exists() => {
            let result: ActionResultContract = read_json(result_path)?;
            if result.final_status == FinalStatus::Succeeded {
                write_timed_out_result(request, workdir, result_path)
            } else {
                Ok(result)
            }
        }
        ExitClassification::TimedOut | ExitClassification::CompletedAfterTimeout(_) => {
            write_timed_out_result(request, workdir, result_path)
        }
        ExitClassification::Completed(status) => {
            let reason = match status.code() {
                Some(code) => format!("exec_exit_{code}"),
                None => "exec_terminated_by_signal".to_string(),
            };
            let result = synthesize_result(request, FinalStatus::Failed, &reason, "failed");
            write_json_atomic(result_path, &result)?;
            write_exec_state(
                workdir,
                &request.execution_id,
                &request.plan.meta.action_id,
                "failed",
                Some(reason),
                "agentd synthesized failure result after abnormal exec exit",
            )?;
            Ok(result)
        }
    }
}

pub fn next_execution_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_nanos();
    format!("exec_{ts}")
}

#[cfg(test)]
mod tests {
    use super::next_execution_id;

    #[test]
    fn execution_id_has_prefix() {
        assert!(next_execution_id().starts_with("exec_"));
    }
}
