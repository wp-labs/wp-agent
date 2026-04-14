use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use wp_agent_shared::fs::{read_json, write_json_atomic};
use wp_agent_shared::paths::{WORKDIR_PLAN_FILE, WORKDIR_RESULT_FILE};

use crate::execution_support::final_state_name;
use crate::process_control::{
    RunningStateStatus, handle_expired_running_state, inspect_running_state,
};
use crate::quarantine::{QuarantineRequest, quarantine_execution};
use crate::recovery::synthesize_recovery_result;
use crate::reporting_pipeline::{ReportingRequest, ensure_local_report};
use crate::state_store::running;

pub(super) fn recover_incomplete_executions_impl(
    state_dir: &Path,
    instance_id: &str,
) -> io::Result<()> {
    let running_dir = state_dir.join("running");
    if !running_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&running_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let mut state: running::RunningExecutionState = match read_json(&path) {
            Ok(state) => state,
            Err(err) => {
                quarantine_execution(QuarantineRequest::unreadable_running(
                    state_dir,
                    path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("unknown-execution"),
                    format!("running state unreadable: {err}"),
                    &path,
                ))?;
                continue;
            }
        };
        let workdir = PathBuf::from(&state.workdir);
        let result_path = workdir.join(WORKDIR_RESULT_FILE);
        let plan = match read_queued_plan(&workdir) {
            Ok(plan) => plan,
            Err(err) => {
                quarantine_running_state(
                    state_dir,
                    &path,
                    &state,
                    format!("running execution plan unavailable: {err}"),
                )?;
                continue;
            }
        };
        let result = if result_path.exists() {
            match read_json(&result_path) {
                Ok(result) => result,
                Err(err) => {
                    quarantine_running_state(
                        state_dir,
                        &path,
                        &state,
                        format!("running execution result unavailable: {err}"),
                    )?;
                    continue;
                }
            }
        } else if execution_is_still_running(&mut state, &path)? {
            continue;
        } else {
            let recovered = synthesize_recovery_result(&state);
            write_json_atomic(&result_path, &recovered)?;
            recovered
        };

        if let Err(err) = ensure_local_report(ReportingRequest {
            state_dir,
            execution_id: &state.execution_id,
            action_id: &state.action_id,
            request_id: &state.request_id,
            plan_digest: &state.plan_digest,
            agent_id: &plan.target.agent_id,
            instance_id,
            final_state: final_state_name(&result),
            result_path: &result_path,
            result: &result,
        }) {
            quarantine_running_state(
                state_dir,
                &path,
                &state,
                format!("running execution report preparation failed: {err}"),
            )?;
            continue;
        }
        running::remove(&path)?;
    }

    Ok(())
}

fn execution_is_still_running(
    state: &mut running::RunningExecutionState,
    running_path: &Path,
) -> io::Result<bool> {
    match inspect_running_state(state)? {
        RunningStateStatus::Active => Ok(true),
        RunningStateStatus::Expired => handle_expired_running_state(state, running_path),
        RunningStateStatus::Inactive => Ok(false),
    }
}

fn read_queued_plan(
    workdir: &Path,
) -> io::Result<wp_agent_contracts::action_plan::ActionPlanContract> {
    read_json(&workdir.join(WORKDIR_PLAN_FILE))
}

fn quarantine_running_state(
    state_dir: &Path,
    running_path: &Path,
    state: &running::RunningExecutionState,
    detail: String,
) -> io::Result<()> {
    quarantine_execution(QuarantineRequest::running_state(
        state_dir,
        state,
        detail,
        running_path,
    ))
}
