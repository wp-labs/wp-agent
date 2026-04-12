//! Local execution queue scheduler.

use std::io;
use std::path::PathBuf;

use wp_agent_contracts::action_plan::{ActionPlanContract, RiskLevel};
use wp_agent_contracts::gateway::ReportActionResult;
use wp_agent_shared::integrity::digest_json;
use wp_agent_shared::time::{after_millis_rfc3339, now_rfc3339};

use crate::local_exec::{LocalExecRequest, execute as execute_local, next_execution_id};
use crate::reporting_pipeline::{ReportingRequest, prepare_local_report};
use crate::state_store::execution_queue::{self, ExecutionQueueItem};
use crate::state_store::running;

#[derive(Debug, Clone)]
pub struct SchedulerRequest {
    pub run_dir: PathBuf,
    pub state_dir: PathBuf,
    pub exec_bin: PathBuf,
    pub plan: ActionPlanContract,
    pub instance_id: String,
}

#[derive(Debug, Clone)]
pub struct SchedulerOutcome {
    pub execution_id: String,
    pub plan_digest: String,
    pub report: ReportActionResult,
}

pub fn submit_local_plan(request: &SchedulerRequest) -> io::Result<SchedulerOutcome> {
    let execution_id = next_execution_id();
    let plan_digest = digest_json(&request.plan)?;
    let deadline_at = Some(after_millis_rfc3339(
        request.plan.constraints.max_total_duration_ms,
    ));

    let queue_path = execution_queue::path_for(&request.state_dir);
    let mut queue = execution_queue::load_or_default(&queue_path)?;
    queue.enqueue(ExecutionQueueItem::new(
        execution_id.clone(),
        request.plan.meta.action_id.clone(),
        plan_digest.clone(),
        request.plan.meta.request_id.clone(),
        100,
        now_rfc3339(),
        deadline_at,
        true,
        Some(risk_level_name(request.plan.constraints.risk_level).to_string()),
    ));
    execution_queue::store(&queue_path, &queue)?;

    let local_result = execute_local(&LocalExecRequest {
        execution_id: execution_id.clone(),
        run_dir: request.run_dir.clone(),
        state_dir: request.state_dir.clone(),
        exec_bin: request.exec_bin.clone(),
        plan_digest: plan_digest.clone(),
        request_id: request.plan.meta.request_id.clone(),
        plan: request.plan.clone(),
    });

    queue.remove(&execution_id);
    execution_queue::store(&queue_path, &queue)?;

    let local_result = local_result?;
    let final_state = final_state_name(&local_result.result);
    let prepared = prepare_local_report(ReportingRequest {
        state_dir: &request.state_dir,
        execution_id: &execution_id,
        action_id: &request.plan.meta.action_id,
        request_id: &request.plan.meta.request_id,
        plan_digest: &plan_digest,
        agent_id: &request.plan.target.agent_id,
        instance_id: &request.instance_id,
        final_state,
        result_path: &local_result.workdir.join("result.json"),
        result: &local_result.result,
    })?;

    let running_path = running::path_for(&request.state_dir, &execution_id);
    running::remove(&running_path)?;

    Ok(SchedulerOutcome {
        execution_id,
        plan_digest,
        report: prepared.envelope,
    })
}

fn final_state_name(
    result: &wp_agent_contracts::action_result::ActionResultContract,
) -> &'static str {
    match result.final_status {
        wp_agent_contracts::action_result::FinalStatus::Succeeded => "succeeded",
        wp_agent_contracts::action_result::FinalStatus::Failed => "failed",
        wp_agent_contracts::action_result::FinalStatus::Cancelled => "cancelled",
        wp_agent_contracts::action_result::FinalStatus::TimedOut => "timed_out",
        wp_agent_contracts::action_result::FinalStatus::Rejected => "rejected",
    }
}

fn risk_level_name(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::R0 => "R0",
        RiskLevel::R1 => "R1",
        RiskLevel::R2 => "R2",
        RiskLevel::R3 => "R3",
    }
}
