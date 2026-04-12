//! Local state validation entrypoints.

use wp_agent_contracts::SCHEMA_VERSION_V1ALPHA1;
use wp_agent_contracts::state_exec::AgentRuntimeState;
use wp_agent_contracts::state_logs::LogStateContract;

use crate::{ValidationError, parse_rfc3339, require_non_empty};

pub fn validate_execution_state(contract: &AgentRuntimeState) -> Result<(), ValidationError> {
    if contract.schema_version != SCHEMA_VERSION_V1ALPHA1 {
        return Err(ValidationError::new("invalid_schema_version"));
    }
    require_non_empty(&contract.agent_id, "missing_runtime_agent_id")?;
    require_non_empty(&contract.instance_id, "missing_runtime_instance_id")?;
    require_non_empty(&contract.version, "missing_runtime_version")?;
    parse_rfc3339(&contract.updated_at, "invalid_runtime_updated_at")?;
    Ok(())
}

pub fn validate_log_state(contract: &LogStateContract) -> Result<(), ValidationError> {
    if contract.schema_version != SCHEMA_VERSION_V1ALPHA1 {
        return Err(ValidationError::new("invalid_schema_version"));
    }
    require_non_empty(&contract.input_id, "missing_input_id")?;
    parse_rfc3339(&contract.updated_at, "invalid_log_state_updated_at")?;

    for file in &contract.files {
        require_non_empty(&file.file_id, "missing_file_id")?;
        require_non_empty(&file.path, "missing_file_path")?;

        if let Some(last_read_at) = &file.last_read_at {
            parse_rfc3339(last_read_at, "invalid_last_read_at")?;
        }
        if let Some(last_commit_point_at) = &file.last_commit_point_at {
            parse_rfc3339(last_commit_point_at, "invalid_last_commit_point_at")?;
        }
    }
    Ok(())
}
