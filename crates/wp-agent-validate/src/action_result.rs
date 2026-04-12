//! `ActionResult` validation entrypoints.

use wp_agent_contracts::API_VERSION_V1;
use wp_agent_contracts::action_result::{ACTION_RESULT_KIND, ActionResultContract};

use crate::{ValidationError, parse_rfc3339, require_non_empty};

pub fn validate_action_result(contract: &ActionResultContract) -> Result<(), ValidationError> {
    if contract.api_version != API_VERSION_V1 {
        return Err(ValidationError::new("invalid_api_version"));
    }
    if contract.kind != ACTION_RESULT_KIND {
        return Err(ValidationError::new("invalid_kind"));
    }

    require_non_empty(&contract.action_id, "missing_action_id")?;
    require_non_empty(&contract.execution_id, "missing_execution_id")?;
    if contract.step_records.is_empty() {
        return Err(ValidationError::new("missing_step_records"));
    }

    for step in &contract.step_records {
        require_non_empty(&step.step_id, "missing_step_id")?;
        if step.attempt == 0 {
            return Err(ValidationError::new("invalid_step_attempt"));
        }

        let started_at = parse_rfc3339(&step.started_at, "invalid_step_started_at")?;
        if let Some(finished_at) = &step.finished_at {
            let finished_at = parse_rfc3339(finished_at, "invalid_step_finished_at")?;
            if finished_at < started_at {
                return Err(ValidationError::new("step_finished_before_started"));
            }
        }
    }

    for item in &contract.outputs.items {
        require_non_empty(&item.name, "missing_output_name")?;
    }

    Ok(())
}
