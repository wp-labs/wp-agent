//! `ActionResult` validation entrypoints.

use wp_agent_contracts::action_result::ActionResultContract;

use crate::ValidationError;

pub fn validate_action_result(contract: &ActionResultContract) -> Result<(), ValidationError> {
    if contract.api_version != "v1alpha1" {
        return Err(ValidationError::new("invalid_api_version"));
    }
    if contract.kind != "ActionResult" {
        return Err(ValidationError::new("invalid_kind"));
    }
    if contract.action_id.is_empty() {
        return Err(ValidationError::new("missing_action_id"));
    }
    if contract.execution_id.is_empty() {
        return Err(ValidationError::new("missing_execution_id"));
    }
    Ok(())
}
