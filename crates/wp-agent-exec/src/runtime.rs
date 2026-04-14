//! Runtime entrypoints.

use std::io;

use wp_agent_contracts::action_result::{
    ActionResultContract, FinalStatus, StepActionRecord, StepStatus,
};
use wp_agent_shared::time::now_rfc3339;
use wp_agent_validate::action_plan::validate_action_plan;

use crate::workdir::{ExecProgressState, ExecutionWorkdir};

pub fn execute(workdir: &ExecutionWorkdir) -> io::Result<ActionResultContract> {
    let runtime = workdir.read_runtime()?;
    let plan = workdir.read_plan()?;

    workdir.write_state(&ExecProgressState {
        execution_id: runtime.execution_id.clone(),
        action_id: plan.meta.action_id.clone(),
        state: "validating".to_string(),
        updated_at: now_rfc3339(),
        step_id: None,
        attempt: None,
        reason_code: None,
        detail: Some("validating action plan".to_string()),
    })?;

    if let Err(err) = validate_action_plan(&plan) {
        let started_at = now_rfc3339();
        let result = ActionResultContract {
            request_id: Some(plan.meta.request_id.clone()),
            exit_reason: Some(err.code.to_string()),
            step_records: vec![StepActionRecord {
                step_id: plan.program.entry.clone(),
                attempt: 1,
                op: None,
                status: StepStatus::Failed,
                started_at: started_at.clone(),
                finished_at: Some(started_at.clone()),
                duration_ms: Some(0),
                error_code: Some(err.code.to_string()),
                stdout_summary: None,
                stderr_summary: None,
                resource_usage: None,
            }],
            started_at: Some(started_at.clone()),
            finished_at: Some(started_at.clone()),
            ..ActionResultContract::new(
                plan.meta.action_id.clone(),
                runtime.execution_id.clone(),
                FinalStatus::Rejected,
            )
        };
        workdir.write_state(&ExecProgressState {
            execution_id: runtime.execution_id,
            action_id: plan.meta.action_id,
            state: "rejected".to_string(),
            updated_at: now_rfc3339(),
            step_id: None,
            attempt: None,
            reason_code: Some(err.code.to_string()),
            detail: Some("action plan validation rejected".to_string()),
        })?;
        return Ok(result);
    }

    let active_step = plan
        .program
        .steps
        .iter()
        .find(|step| step.id == plan.program.entry)
        .map(|step| step.id.clone());

    workdir.write_state(&ExecProgressState {
        execution_id: runtime.execution_id.clone(),
        action_id: plan.meta.action_id.clone(),
        state: "running".to_string(),
        updated_at: now_rfc3339(),
        step_id: active_step,
        attempt: Some(1),
        reason_code: None,
        detail: Some("executing skeleton plan".to_string()),
    })?;

    let started_at = now_rfc3339();
    let finished_at = now_rfc3339();
    let step_records = plan
        .program
        .steps
        .iter()
        .map(|step| StepActionRecord {
            step_id: step.id.clone(),
            attempt: 1,
            op: step.op.clone(),
            status: StepStatus::Succeeded,
            started_at: started_at.clone(),
            finished_at: Some(finished_at.clone()),
            duration_ms: Some(0),
            error_code: None,
            stdout_summary: None,
            stderr_summary: None,
            resource_usage: None,
        })
        .collect();

    Ok(ActionResultContract {
        request_id: Some(plan.meta.request_id),
        step_records,
        started_at: Some(started_at),
        finished_at: Some(finished_at),
        ..ActionResultContract::new(
            plan.meta.action_id,
            runtime.execution_id,
            FinalStatus::Succeeded,
        )
    })
}
