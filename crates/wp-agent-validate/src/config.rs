//! Config validation entrypoints.

use wp_agent_contracts::agent_config::AgentConfigContract;

use crate::ValidationError;

pub fn validate_config(contract: &AgentConfigContract) -> Result<(), ValidationError> {
    if contract.schema_version != "v1alpha1" {
        return Err(ValidationError::new("invalid_schema_version"));
    }
    if contract.control_plane.enabled && contract.control_plane.endpoint.is_none() {
        return Err(ValidationError::new("missing_control_plane_endpoint"));
    }
    if contract.paths.root_dir.is_empty()
        || contract.paths.run_dir.is_empty()
        || contract.paths.state_dir.is_empty()
        || contract.paths.log_dir.is_empty()
    {
        return Err(ValidationError::new("missing_required_paths"));
    }
    Ok(())
}
