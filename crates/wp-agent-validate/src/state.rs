//! Local state validation entrypoints.

use wp_agent_contracts::SCHEMA_VERSION_V1;
use wp_agent_contracts::state_exec::AgentRuntimeState;

use crate::{ValidationError, parse_rfc3339, require_non_empty};

pub fn validate_execution_state(contract: &AgentRuntimeState) -> Result<(), ValidationError> {
    if contract.schema_version != SCHEMA_VERSION_V1 {
        return Err(ValidationError::new("invalid_schema_version"));
    }
    require_non_empty(&contract.agent_id, "missing_runtime_agent_id")?;
    require_non_empty(&contract.instance_id, "missing_runtime_instance_id")?;
    require_non_empty(&contract.version, "missing_runtime_version")?;
    parse_rfc3339(&contract.updated_at, "invalid_runtime_updated_at")?;
    Ok(())
}
