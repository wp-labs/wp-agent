//! Local state validation entrypoints.

use wp_agent_contracts::state_exec::AgentRuntimeState;
use wp_agent_contracts::state_logs::LogStateContract;

use crate::ValidationError;

pub fn validate_execution_state(contract: &AgentRuntimeState) -> Result<(), ValidationError> {
    if contract.schema_version != "v1alpha1" {
        return Err(ValidationError::new("invalid_schema_version"));
    }
    if contract.agent_id.is_empty() || contract.instance_id.is_empty() {
        return Err(ValidationError::new("missing_runtime_identity"));
    }
    Ok(())
}

pub fn validate_log_state(contract: &LogStateContract) -> Result<(), ValidationError> {
    if contract.schema_version != "v1alpha1" {
        return Err(ValidationError::new("invalid_schema_version"));
    }
    if contract.input_id.is_empty() {
        return Err(ValidationError::new("missing_input_id"));
    }
    Ok(())
}
