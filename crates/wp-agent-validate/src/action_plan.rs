//! `ActionPlan` validation entrypoints.

use wp_agent_contracts::action_plan::ActionPlanContract;

use crate::ValidationError;

pub fn validate_action_plan(contract: &ActionPlanContract) -> Result<(), ValidationError> {
    if contract.api_version != "v1alpha1" {
        return Err(ValidationError::new("invalid_api_version"));
    }
    if contract.kind != "ActionPlan" {
        return Err(ValidationError::new("invalid_kind"));
    }
    if contract.meta.action_id.is_empty() {
        return Err(ValidationError::new("missing_action_id"));
    }
    if contract.target.agent_id.is_empty() {
        return Err(ValidationError::new("missing_target_agent_id"));
    }
    if contract.program.entry.is_empty() {
        return Err(ValidationError::new("missing_program_entry"));
    }
    if contract.program.steps.is_empty() {
        return Err(ValidationError::new("missing_program_steps"));
    }
    Ok(())
}
