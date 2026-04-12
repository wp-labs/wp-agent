//! Local execution controller for `wp-agent-exec`.

use std::fs::{self, File};
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use wp_agent_contracts::action_plan::ActionPlanContract;
use wp_agent_contracts::action_result::{ActionResultContract, FinalStatus};
use wp_agent_shared::fs::{read_json, write_json_atomic};
use wp_agent_shared::paths::{
    ACTIONS_DIR, WORKDIR_PLAN_FILE, WORKDIR_RESULT_FILE, WORKDIR_RUNTIME_FILE,
};
use wp_agent_shared::time::{after_millis_rfc3339, now_rfc3339};

use crate::state_store::running;

#[derive(Debug, Clone)]
pub struct LocalExecRequest {
    pub execution_id: String,
    pub run_dir: PathBuf,
    pub state_dir: PathBuf,
    pub exec_bin: PathBuf,
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
        .stdout(Stdio::from(stdout_log))
        .stderr(Stdio::from(stderr_log))
        .spawn()?;

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
        started_at.clone(),
        runtime.deadline_at.clone(),
        None,
        Some(1),
        None,
        None,
        started_at,
    );
    running::store(&running_path, &running_state)?;

    let exit_status = wait_for_child(&mut child, request.plan.constraints.max_total_duration_ms)?;

    let result_path = workdir.join(WORKDIR_RESULT_FILE);
    let result: ActionResultContract = read_json(&result_path)?;

    let finished_state = running::RunningExecutionState::new(
        request.execution_id.clone(),
        request.plan.meta.action_id.clone(),
        request.plan_digest.clone(),
        request.request_id.clone(),
        final_state_name(&result).to_string(),
        workdir.display().to_string(),
        child.id().into(),
        running_state.started_at,
        runtime.deadline_at,
        None,
        Some(1),
        None,
        None,
        now_rfc3339(),
    );
    running::store(&running_path, &finished_state)?;

    if !exit_status.success() && result.final_status == FinalStatus::Succeeded {
        return Err(io::Error::other(
            "exec process exited non-zero with succeeded result",
        ));
    }

    Ok(LocalExecOutcome {
        execution_id: request.execution_id.clone(),
        workdir,
        result,
    })
}

fn wait_for_child(child: &mut std::process::Child, timeout_ms: u64) -> io::Result<ExitStatus> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(1));
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            child.kill()?;
            return child.wait();
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn final_state_name(result: &ActionResultContract) -> &'static str {
    match result.final_status {
        FinalStatus::Succeeded => "succeeded",
        FinalStatus::Failed => "failed",
        FinalStatus::Cancelled => "cancelled",
        FinalStatus::TimedOut => "timed_out",
        FinalStatus::Rejected => "rejected",
    }
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
