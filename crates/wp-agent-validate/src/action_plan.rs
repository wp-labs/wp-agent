//! `ActionPlan` validation entrypoints.

use std::collections::BTreeSet;

use wp_agent_contracts::API_VERSION_V1ALPHA1;
use wp_agent_contracts::action_plan::{ACTION_PLAN_KIND, ActionPlanContract};

use crate::{ValidationError, parse_rfc3339, require_non_empty};

pub fn validate_action_plan(contract: &ActionPlanContract) -> Result<(), ValidationError> {
    if contract.api_version != API_VERSION_V1ALPHA1 {
        return Err(ValidationError::new("invalid_api_version"));
    }
    if contract.kind != ACTION_PLAN_KIND {
        return Err(ValidationError::new("invalid_kind"));
    }

    require_non_empty(&contract.meta.action_id, "missing_action_id")?;
    require_non_empty(&contract.meta.request_id, "missing_request_id")?;
    require_non_empty(&contract.meta.tenant_id, "missing_tenant_id")?;
    require_non_empty(&contract.meta.environment_id, "missing_environment_id")?;
    if contract.meta.plan_version == 0 {
        return Err(ValidationError::new("invalid_plan_version"));
    }

    let compiled_at = parse_rfc3339(&contract.meta.compiled_at, "invalid_compiled_at")?;
    let expires_at = parse_rfc3339(&contract.meta.expires_at, "invalid_expires_at")?;
    if expires_at <= compiled_at {
        return Err(ValidationError::new("expired_or_non_increasing_window"));
    }

    require_non_empty(&contract.target.agent_id, "missing_target_agent_id")?;
    require_non_empty(&contract.target.node_id, "missing_target_node_id")?;
    require_non_empty(&contract.target.platform, "missing_target_platform")?;
    require_non_empty(&contract.target.arch, "missing_target_arch")?;

    require_non_empty(
        &contract.constraints.requested_by,
        "missing_constraints_requested_by",
    )?;
    if contract.constraints.max_total_duration_ms == 0 {
        return Err(ValidationError::new("invalid_max_total_duration_ms"));
    }
    if contract.constraints.step_timeout_default_ms == 0 {
        return Err(ValidationError::new("invalid_step_timeout_default_ms"));
    }
    if contract.constraints.step_timeout_default_ms > contract.constraints.max_total_duration_ms {
        return Err(ValidationError::new("step_timeout_exceeds_total_duration"));
    }
    require_non_empty(
        &contract.constraints.execution_profile,
        "missing_execution_profile",
    )?;

    require_non_empty(&contract.program.entry, "missing_program_entry")?;
    if contract.program.steps.is_empty() {
        return Err(ValidationError::new("missing_program_steps"));
    }

    let mut step_ids = BTreeSet::new();
    let mut entry_found = false;
    for step in &contract.program.steps {
        require_non_empty(&step.id, "missing_step_id")?;
        require_non_empty(&step.kind, "missing_step_kind")?;

        if step.kind == "invoke" {
            let op = step.op.as_deref().unwrap_or_default();
            require_non_empty(op, "missing_invoke_op")?;
        }

        if !step_ids.insert(step.id.as_str()) {
            return Err(ValidationError::new("duplicate_step_id"));
        }
        if step.id == contract.program.entry {
            entry_found = true;
        }
    }

    if !entry_found {
        return Err(ValidationError::new("program_entry_not_found"));
    }

    Ok(())
}
