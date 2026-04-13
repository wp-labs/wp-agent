//! `wp-agentd` runtime loop and recovery helpers.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use wp_agent_contracts::agent_config::AgentConfigContract;
use wp_agent_shared::fs::{read_json, write_json_atomic};
use wp_agent_shared::paths::{REPORT_ENVELOPE_SUFFIX, WORKDIR_PLAN_FILE, WORKDIR_RESULT_FILE};
use wp_agent_shared::time::now_rfc3339;

use crate::execution_support::final_state_name;
use crate::process_control::running_state_is_active;
use crate::quarantine::{QuarantineRequest, quarantine_execution};
use crate::recovery::synthesize_recovery_result;
use crate::reporting_pipeline::{ReportingRequest, ensure_local_report};
use crate::scheduler;
use crate::self_observability::{HealthState, RuntimeHealthSnapshot, emit};
use crate::state_store::{agent_runtime, execution_queue, running};

pub struct DaemonLoop<'a> {
    pub config: &'a AgentConfigContract,
    pub exec_bin: &'a Path,
}

pub fn run_forever(loop_ctx: DaemonLoop<'_>) -> io::Result<()> {
    let sleep_interval = Duration::from_millis(250);
    loop {
        let snapshot = run_once(&loop_ctx)?;
        emit(&snapshot);
        thread::sleep(sleep_interval);
    }
}

pub fn run_once(loop_ctx: &DaemonLoop<'_>) -> io::Result<RuntimeHealthSnapshot> {
    let run_dir = Path::new(&loop_ctx.config.paths.run_dir);
    let state_dir = Path::new(&loop_ctx.config.paths.state_dir);

    recover_incomplete_executions(state_dir, &instance_id(loop_ctx.config))?;
    let drained = scheduler::drain_next(&scheduler::DrainRequest {
        run_dir: run_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        exec_bin: loop_ctx.exec_bin.to_path_buf(),
        instance_id: instance_id(loop_ctx.config),
        cancel_grace_ms: loop_ctx.config.execution.cancel_grace_ms,
        stdout_limit_bytes: loop_ctx.config.execution.default_stdout_limit_bytes,
        stderr_limit_bytes: loop_ctx.config.execution.default_stderr_limit_bytes,
    })?;

    let queue = execution_queue::load_or_default(&execution_queue::path_for(state_dir))?;
    let running_count = count_running_entries(state_dir)?;
    let reporting_count = count_reporting_entries(state_dir)?;
    let health = RuntimeHealthSnapshot {
        state: if drained || running_count > 0 || reporting_count > 0 || !queue.items.is_empty() {
            HealthState::Active
        } else {
            HealthState::Idle
        },
        queue_depth: queue.items.len(),
        running_count,
        reporting_count,
        updated_at: now_rfc3339(),
    };

    let runtime_path = agent_runtime::path_for(state_dir);
    let mut runtime_state = agent_runtime::load_or_default(&runtime_path)?;
    runtime_state.updated_at = health.updated_at.clone();
    agent_runtime::store(&runtime_path, &runtime_state)?;

    Ok(health)
}

pub fn recover_incomplete_executions(state_dir: &Path, instance_id: &str) -> io::Result<()> {
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
    running_state_is_active(state, running_path)
}

fn count_running_entries(state_dir: &Path) -> io::Result<usize> {
    let running_dir = state_dir.join("running");
    if !running_dir.exists() {
        return Ok(0);
    }

    let mut count = 0usize;
    for entry in fs::read_dir(running_dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|ext| ext.to_str()) == Some("json") {
            count += 1;
        }
    }
    Ok(count)
}

fn count_reporting_entries(state_dir: &Path) -> io::Result<usize> {
    let reporting_dir = state_dir.join("reporting");
    if !reporting_dir.exists() {
        return Ok(0);
    }

    let mut count = 0usize;
    for entry in fs::read_dir(reporting_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if file_name.ends_with(REPORT_ENVELOPE_SUFFIX) {
            continue;
        }
        count += 1;
    }
    Ok(count)
}

fn read_queued_plan(
    workdir: &Path,
) -> io::Result<wp_agent_contracts::action_plan::ActionPlanContract> {
    read_json(&workdir.join(WORKDIR_PLAN_FILE))
}

fn instance_id(config: &AgentConfigContract) -> String {
    config
        .agent
        .instance_name
        .clone()
        .unwrap_or_else(|| "local-instance".to_string())
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
