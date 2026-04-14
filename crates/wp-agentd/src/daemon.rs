//! `wp-agentd` runtime loop and recovery helpers.

use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;

use wp_agent_contracts::agent_config::AgentConfigContract;
use wp_agent_shared::time::now_rfc3339;

use crate::scheduler;
use crate::self_observability::{HealthState, RuntimeHealthSnapshot, emit};
use crate::state_store::{agent_runtime, execution_queue};

#[path = "daemon_recovery.rs"]
mod recovery_support;
#[path = "daemon_runtime_state.rs"]
mod runtime_state_support;
#[path = "daemon_telemetry.rs"]
mod telemetry_support;

use recovery_support::recover_incomplete_executions_impl;
use runtime_state_support::{
    count_reporting_entries, count_running_entries, emit_telemetry_failures, instance_id,
};
use telemetry_support::process_telemetry_inputs;

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
    let instance_id = instance_id(loop_ctx.config);
    let telemetry_tick = process_telemetry_inputs(loop_ctx.config);
    emit_telemetry_failures(&telemetry_tick.failures);
    let telemetry_active = telemetry_tick.is_active();

    recover_incomplete_executions(state_dir, &instance_id)?;
    let drained = scheduler::drain_next(&scheduler::DrainRequest {
        run_dir: run_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        exec_bin: loop_ctx.exec_bin.to_path_buf(),
        instance_id,
        cancel_grace_ms: loop_ctx.config.execution.cancel_grace_ms,
        stdout_limit_bytes: loop_ctx.config.execution.default_stdout_limit_bytes,
        stderr_limit_bytes: loop_ctx.config.execution.default_stderr_limit_bytes,
    })?;

    let queue = execution_queue::load_or_default(&execution_queue::path_for(state_dir))?;
    let running_count = count_running_entries(state_dir)?;
    let reporting_count = count_reporting_entries(state_dir)?;
    let health = RuntimeHealthSnapshot {
        state: if telemetry_active
            || drained
            || running_count > 0
            || reporting_count > 0
            || !queue.items.is_empty()
        {
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
    recover_incomplete_executions_impl(state_dir, instance_id)
}
