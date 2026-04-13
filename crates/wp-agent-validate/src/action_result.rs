//! `ActionResult` validation entrypoints.

use wp_agent_contracts::API_VERSION_V1;
use wp_agent_contracts::action_result::{
    ACTION_RESULT_KIND, ActionResultContract, FinalStatus, StepStatus,
};

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
    if let Some(request_id) = &contract.request_id {
        require_non_empty(request_id, "invalid_request_id")?;
    }
    if contract.step_records.is_empty() {
        return Err(ValidationError::new("missing_step_records"));
    }

    let started_at = contract
        .started_at
        .as_deref()
        .map(|value| parse_rfc3339(value, "invalid_started_at"))
        .transpose()?;
    let finished_at = contract
        .finished_at
        .as_deref()
        .map(|value| parse_rfc3339(value, "invalid_finished_at"))
        .transpose()?;
    if let (Some(started_at), Some(finished_at)) = (started_at, finished_at) {
        if finished_at < started_at {
            return Err(ValidationError::new("finished_before_started"));
        }
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

    validate_final_status_consistency(contract)?;

    Ok(())
}

fn validate_final_status_consistency(
    contract: &ActionResultContract,
) -> Result<(), ValidationError> {
    match contract.final_status {
        FinalStatus::Succeeded => {
            if contract.exit_reason.is_some() {
                return Err(ValidationError::new("succeeded_result_has_exit_reason"));
            }
            if contract
                .step_records
                .iter()
                .any(|step| !matches!(step.status, StepStatus::Succeeded | StepStatus::Skipped))
            {
                return Err(ValidationError::new(
                    "succeeded_result_has_non_success_step",
                ));
            }
        }
        FinalStatus::Rejected => {
            if contract
                .step_records
                .iter()
                .any(|step| matches!(step.status, StepStatus::Succeeded))
            {
                return Err(ValidationError::new("rejected_result_has_success_step"));
            }
        }
        FinalStatus::Failed => {
            if contract
                .step_records
                .iter()
                .all(|step| !matches!(step.status, StepStatus::Failed))
            {
                return Err(ValidationError::new("failed_result_has_no_failed_step"));
            }
        }
        FinalStatus::TimedOut => {
            if contract
                .step_records
                .iter()
                .all(|step| !matches!(step.status, StepStatus::TimedOut))
            {
                return Err(ValidationError::new(
                    "timed_out_result_has_no_timed_out_step",
                ));
            }
        }
        FinalStatus::Cancelled => {
            if contract
                .step_records
                .iter()
                .all(|step| !matches!(step.status, StepStatus::Cancelled))
            {
                return Err(ValidationError::new(
                    "cancelled_result_has_no_cancelled_step",
                ));
            }
        }
    }

    Ok(())
}
