//! Result writer helpers.

use std::io;

use wp_agent_contracts::action_result::ActionResultContract;
use wp_agent_shared::time::now_rfc3339;

use crate::workdir::{ExecProgressState, ExecutionWorkdir};

pub fn write(workdir: &ExecutionWorkdir, result: &ActionResultContract) -> io::Result<()> {
    workdir.write_result(result)?;
    workdir.write_state(&ExecProgressState {
        execution_id: result.execution_id.clone(),
        action_id: result.action_id.clone(),
        state: map_final_state(result).to_string(),
        updated_at: now_rfc3339(),
        step_id: None,
        attempt: None,
        reason_code: result.exit_reason.clone(),
        detail: Some("final result persisted".to_string()),
    })?;
    Ok(())
}

fn map_final_state(result: &ActionResultContract) -> &'static str {
    match result.final_status {
        wp_agent_contracts::action_result::FinalStatus::Succeeded => "succeeded",
        wp_agent_contracts::action_result::FinalStatus::Failed => "failed",
        wp_agent_contracts::action_result::FinalStatus::Cancelled => "cancelled",
        wp_agent_contracts::action_result::FinalStatus::TimedOut => "timed_out",
        wp_agent_contracts::action_result::FinalStatus::Rejected => "rejected",
    }
}
