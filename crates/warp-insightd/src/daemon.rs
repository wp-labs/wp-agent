//! `warp-insightd` runtime loop and recovery helpers.

use std::collections::BTreeSet;
use std::io;
use std::path::Path;
use std::time::Duration;

use warp_insight_contracts::agent_config::AgentConfigContract;
use warp_insight_shared::time::now_rfc3339;

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
    count_reporting_entries, count_running_entries, emit_telemetry_failure,
    emit_telemetry_failures, failure_signatures, filter_new_failures, instance_id,
};
use telemetry_support::process_telemetry_inputs;

pub struct DaemonLoop<'a> {
    pub config: &'a AgentConfigContract,
    pub exec_bin: &'a Path,
}

pub async fn run_forever_async(loop_ctx: DaemonLoop<'_>) -> io::Result<()> {
    let sleep_interval = Duration::from_millis(250);
    let mut previous_telemetry_failures = BTreeSet::new();
    loop {
        let snapshot =
            run_once_with_failure_cache(&loop_ctx, Some(&mut previous_telemetry_failures)).await?;
        emit(&snapshot);
        tokio::time::sleep(sleep_interval).await;
    }
}

pub async fn run_once_async(loop_ctx: &DaemonLoop<'_>) -> io::Result<RuntimeHealthSnapshot> {
    run_once_with_failure_cache(loop_ctx, None).await
}

async fn run_once_with_failure_cache(
    loop_ctx: &DaemonLoop<'_>,
    previous_telemetry_failures: Option<&mut BTreeSet<String>>,
) -> io::Result<RuntimeHealthSnapshot> {
    let run_dir = Path::new(&loop_ctx.config.paths.run_dir);
    let state_dir = Path::new(&loop_ctx.config.paths.state_dir);
    let instance_id = instance_id(loop_ctx.config);
    let telemetry_tick = process_telemetry_inputs(loop_ctx.config).await;
    if let Some(previous) = previous_telemetry_failures {
        for failure in filter_new_failures(&telemetry_tick.failures, previous) {
            emit_telemetry_failure(failure);
        }
        *previous = failure_signatures(&telemetry_tick.failures);
    } else {
        emit_telemetry_failures(&telemetry_tick.failures);
    }
    let telemetry_active = telemetry_tick.is_active();

    recover_incomplete_executions(state_dir, &instance_id)?;
    let drained = scheduler::drain_next_async(&scheduler::DrainRequest {
        run_dir: run_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        exec_bin: loop_ctx.exec_bin.to_path_buf(),
        instance_id,
        cancel_grace_ms: loop_ctx.config.execution.cancel_grace_ms,
        stdout_limit_bytes: loop_ctx.config.execution.default_stdout_limit_bytes,
        stderr_limit_bytes: loop_ctx.config.execution.default_stderr_limit_bytes,
    })
    .await?;

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

pub fn run_once(loop_ctx: &DaemonLoop<'_>) -> io::Result<RuntimeHealthSnapshot> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run_once_async(loop_ctx))
}

pub fn recover_incomplete_executions(state_dir: &Path, instance_id: &str) -> io::Result<()> {
    recover_incomplete_executions_impl(state_dir, instance_id)
}
