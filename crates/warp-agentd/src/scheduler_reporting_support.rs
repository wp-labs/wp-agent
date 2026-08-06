use std::io;
use std::path::Path;

use warp_insight_contracts::action_plan::ActionPlanContract;
use warp_insight_contracts::action_result::ActionResultContract;
use warp_insight_contracts::gateway::ReportActionResult;
use warp_insight_shared::fs::{read_json, write_json_atomic};
use warp_insight_shared::paths::{WORKDIR_PLAN_FILE, WORKDIR_RESULT_FILE};

use crate::execution_support::final_state_name;
use crate::local_exec::LocalExecOutcome;
use crate::recovery::synthesize_recovery_result;
use crate::reporting_pipeline::{
    PreparedReport, ReportingRequest, ensure_local_report, load_complete_local_report,
    prepare_local_report,
};
use crate::scheduler::{DrainOutcome, DrainRequest};
use crate::state_store::execution_queue::ExecutionQueueItem;
use crate::state_store::running;

use super::QueueHeadContext;

pub(super) fn read_queued_plan(workdir: &Path) -> io::Result<ActionPlanContract> {
    read_json(&workdir.join(WORKDIR_PLAN_FILE))
}

pub(super) fn prepare_queue_head_report(
    request: &DrainRequest,
    item: &ExecutionQueueItem,
    plan: &ActionPlanContract,
    local_result: &LocalExecOutcome,
) -> io::Result<PreparedReport> {
    prepare_local_report(ReportingRequest {
        state_dir: &request.state_dir,
        execution_id: &item.execution_id,
        action_id: &item.action_id,
        request_id: &item.request_id,
        plan_digest: &item.plan_digest,
        agent_id: &plan.target.agent_id,
        instance_id: &request.instance_id,
        final_state: final_state_name(&local_result.result),
        result_path: &local_result.workdir.join(WORKDIR_RESULT_FILE),
        result: &local_result.result,
    })
}

pub(super) fn recover_stale_execution(
    request: &DrainRequest,
    item: &ExecutionQueueItem,
    head: &QueueHeadContext,
    running_state: &running::RunningExecutionState,
) -> io::Result<DrainOutcome> {
    let result_path = head.workdir.join(WORKDIR_RESULT_FILE);
    let recovered = synthesize_recovery_result(running_state);
    write_json_atomic(&result_path, &recovered)?;
    let prepared = ensure_local_report(ReportingRequest {
        state_dir: &request.state_dir,
        execution_id: &item.execution_id,
        action_id: &item.action_id,
        request_id: &item.request_id,
        plan_digest: &item.plan_digest,
        agent_id: &head.plan.target.agent_id,
        instance_id: &request.instance_id,
        final_state: final_state_name(&recovered),
        result_path: &result_path,
        result: &recovered,
    })?;
    Ok(drain_outcome(item, prepared.envelope))
}

pub(super) fn reconcile_completed_execution(
    request: &DrainRequest,
    item: &ExecutionQueueItem,
    plan: &ActionPlanContract,
    workdir: &Path,
) -> io::Result<Option<DrainOutcome>> {
    let result_path = workdir.join(WORKDIR_RESULT_FILE);
    if let Some(prepared) = load_complete_local_report(&request.state_dir, &item.execution_id)? {
        return Ok(Some(drain_outcome(item, prepared.envelope)));
    }

    if !result_path.exists() {
        return Ok(None);
    }

    let result: ActionResultContract = read_json(&result_path)?;
    let final_state = final_state_name(&result);
    let prepared = ensure_local_report(ReportingRequest {
        state_dir: &request.state_dir,
        execution_id: &item.execution_id,
        action_id: &item.action_id,
        request_id: &item.request_id,
        plan_digest: &item.plan_digest,
        agent_id: &plan.target.agent_id,
        instance_id: &request.instance_id,
        final_state,
        result_path: &result_path,
        result: &result,
    })?;

    Ok(Some(drain_outcome(item, prepared.envelope)))
}

fn drain_outcome(item: &ExecutionQueueItem, report: ReportActionResult) -> DrainOutcome {
    DrainOutcome {
        execution_id: item.execution_id.clone(),
        plan_digest: item.plan_digest.clone(),
        report,
    }
}
