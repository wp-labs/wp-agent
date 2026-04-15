//! Gateway envelope validation entrypoints.

use warp_insight_contracts::API_VERSION_V1;
use warp_insight_contracts::gateway::{
    ACTION_PLAN_ACK_KIND, AckStatus, ActionPlanAck, DISPATCH_ACTION_PLAN_KIND, DispatchActionPlan,
    REPORT_ACTION_RESULT_KIND, ReportActionResult,
};

use crate::action_plan::validate_action_plan;
use crate::action_result::validate_action_result;
use crate::{ValidationError, parse_rfc3339, require_non_empty};

pub fn validate_dispatch_action_plan(contract: &DispatchActionPlan) -> Result<(), ValidationError> {
    if contract.api_version != API_VERSION_V1 {
        return Err(ValidationError::new("invalid_api_version"));
    }
    if contract.kind != DISPATCH_ACTION_PLAN_KIND {
        return Err(ValidationError::new("invalid_kind"));
    }
    require_non_empty(&contract.dispatch_id, "missing_dispatch_id")?;
    validate_action_plan(&contract.plan)?;
    Ok(())
}

pub fn validate_action_plan_ack(contract: &ActionPlanAck) -> Result<(), ValidationError> {
    if contract.api_version != API_VERSION_V1 {
        return Err(ValidationError::new("invalid_api_version"));
    }
    if contract.kind != ACTION_PLAN_ACK_KIND {
        return Err(ValidationError::new("invalid_kind"));
    }

    require_non_empty(&contract.dispatch_id, "missing_dispatch_id")?;
    require_non_empty(&contract.action_id, "missing_action_id")?;
    require_non_empty(&contract.plan_digest, "missing_plan_digest")?;
    require_non_empty(&contract.agent_id, "missing_agent_id")?;
    require_non_empty(&contract.instance_id, "missing_instance_id")?;

    let received_at = parse_rfc3339(&contract.received_at, "invalid_received_at")?;
    let acknowledged_at = parse_rfc3339(&contract.acknowledged_at, "invalid_acknowledged_at")?;
    if acknowledged_at < received_at {
        return Err(ValidationError::new("acknowledged_before_received"));
    }

    if let Some(reason_code) = &contract.reason_code {
        require_non_empty(reason_code, "invalid_reason_code")?;
    }
    if let Some(reason_message) = &contract.reason_message {
        require_non_empty(reason_message, "invalid_reason_message")?;
    }

    match contract.ack_status {
        AckStatus::Accepted => {
            let execution_id = contract.execution_id.as_deref().unwrap_or_default();
            require_non_empty(execution_id, "missing_execution_id")?;
            if contract.queue_position.is_some() {
                return Err(ValidationError::new(
                    "queue_position_not_allowed_for_accepted",
                ));
            }
        }
        AckStatus::Queued => {
            let execution_id = contract.execution_id.as_deref().unwrap_or_default();
            require_non_empty(execution_id, "missing_execution_id")?;
            if contract.queue_position.is_none() {
                return Err(ValidationError::new("missing_queue_position"));
            }
        }
        AckStatus::Rejected | AckStatus::Duplicate | AckStatus::Stale | AckStatus::Busy => {
            if contract.queue_position.is_some() {
                return Err(ValidationError::new(
                    "queue_position_only_allowed_for_queued",
                ));
            }
        }
    }

    Ok(())
}

pub fn validate_report_action_result(contract: &ReportActionResult) -> Result<(), ValidationError> {
    if contract.api_version != API_VERSION_V1 {
        return Err(ValidationError::new("invalid_api_version"));
    }
    if contract.kind != REPORT_ACTION_RESULT_KIND {
        return Err(ValidationError::new("invalid_kind"));
    }

    require_non_empty(&contract.report_id, "missing_report_id")?;
    if let Some(dispatch_id) = &contract.dispatch_id {
        require_non_empty(dispatch_id, "invalid_dispatch_id")?;
    }
    require_non_empty(&contract.action_id, "missing_action_id")?;
    require_non_empty(&contract.execution_id, "missing_execution_id")?;
    require_non_empty(&contract.plan_digest, "missing_plan_digest")?;
    require_non_empty(&contract.agent_id, "missing_agent_id")?;
    require_non_empty(&contract.instance_id, "missing_instance_id")?;
    if contract.report_attempt == 0 {
        return Err(ValidationError::new("invalid_report_attempt"));
    }
    let reported_at = parse_rfc3339(&contract.reported_at, "invalid_reported_at")?;

    require_non_empty(
        &contract.result_attestation.result_digest,
        "missing_result_digest",
    )?;
    require_non_empty(&contract.result_attestation.signature, "missing_signature")?;
    require_non_empty(&contract.result_attestation.issued_by, "missing_issued_by")?;
    let attested_at = parse_rfc3339(
        &contract.result_attestation.attested_at,
        "invalid_attested_at",
    )?;
    if attested_at > reported_at {
        return Err(ValidationError::new("attested_after_reported"));
    }

    validate_action_result(&contract.result)?;
    if contract.final_status != contract.result.final_status {
        return Err(ValidationError::new("mismatched_final_status"));
    }
    if contract.action_id != contract.result.action_id {
        return Err(ValidationError::new("mismatched_action_id"));
    }
    if contract.execution_id != contract.result.execution_id {
        return Err(ValidationError::new("mismatched_execution_id"));
    }

    Ok(())
}
