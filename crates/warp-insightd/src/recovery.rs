//! Shared recovery helpers for incomplete executions.

use warp_insight_contracts::action_result::{
    ActionOutputs, ActionResultContract, FinalStatus, StepActionRecord, StepStatus,
};
use warp_insight_shared::time::now_rfc3339;

use crate::state_store::running;

pub(crate) fn synthesize_recovery_result(
    state: &running::RunningExecutionState,
) -> ActionResultContract {
    let timestamp = now_rfc3339();
    ActionResultContract {
        request_id: Some(state.request_id.clone()),
        exit_reason: Some("agentd_recovered_incomplete_execution".to_string()),
        step_records: vec![StepActionRecord {
            step_id: state
                .current_step_id
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            attempt: state.attempt.unwrap_or(1),
            op: None,
            status: StepStatus::Failed,
            started_at: state.started_at.clone(),
            finished_at: Some(timestamp.clone()),
            duration_ms: None,
            error_code: Some("agentd_recovered_incomplete_execution".to_string()),
            stdout_summary: None,
            stderr_summary: None,
            resource_usage: None,
        }],
        outputs: ActionOutputs::default(),
        started_at: Some(state.started_at.clone()),
        finished_at: Some(timestamp),
        ..ActionResultContract::new(
            state.action_id.clone(),
            state.execution_id.clone(),
            FinalStatus::Failed,
        )
    }
}
