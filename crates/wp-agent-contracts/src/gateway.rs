//! Gateway envelope contract types.

use serde::{Deserialize, Serialize};

use crate::API_VERSION_V1ALPHA1;
use crate::action_plan::ActionPlanContract;
use crate::action_result::ActionResultContract;

pub const DISPATCH_ACTION_PLAN_KIND: &str = "dispatch_action_plan";
pub const ACTION_PLAN_ACK_KIND: &str = "action_plan_ack";
pub const REPORT_ACTION_RESULT_KIND: &str = "report_action_result";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentHello {
    pub agent_id: String,
    pub instance_id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchActionPlan {
    pub api_version: String,
    pub kind: String,
    pub dispatch_id: String,
    pub plan: ActionPlanContract,
}

impl DispatchActionPlan {
    pub fn new(dispatch_id: String, plan: ActionPlanContract) -> Self {
        Self {
            api_version: API_VERSION_V1ALPHA1.to_string(),
            kind: DISPATCH_ACTION_PLAN_KIND.to_string(),
            dispatch_id,
            plan,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPlanAck {
    pub api_version: String,
    pub kind: String,
    pub dispatch_id: String,
    pub action_id: String,
    pub plan_digest: String,
    pub agent_id: String,
    pub instance_id: String,
    pub execution_id: String,
    pub ack_status: AckStatus,
    pub reason_code: Option<String>,
    pub reason_message: Option<String>,
    pub queue_position: Option<u64>,
    pub received_at: String,
    pub acknowledged_at: String,
}

impl ActionPlanAck {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dispatch_id: String,
        action_id: String,
        plan_digest: String,
        agent_id: String,
        instance_id: String,
        execution_id: String,
        ack_status: AckStatus,
        received_at: String,
        acknowledged_at: String,
    ) -> Self {
        Self {
            api_version: API_VERSION_V1ALPHA1.to_string(),
            kind: ACTION_PLAN_ACK_KIND.to_string(),
            dispatch_id,
            action_id,
            plan_digest,
            agent_id,
            instance_id,
            execution_id,
            ack_status,
            reason_code: None,
            reason_message: None,
            queue_position: None,
            received_at,
            acknowledged_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportActionResult {
    pub api_version: String,
    pub report_id: String,
    pub kind: String,
    pub dispatch_id: Option<String>,
    pub action_id: String,
    pub report_attempt: u32,
    pub final_status: String,
    pub execution_id: String,
    pub plan_digest: String,
    pub agent_id: String,
    pub instance_id: String,
    pub result_attestation: ResultAttestation,
    pub reported_at: String,
    pub result: ActionResultContract,
}

impl ReportActionResult {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        report_id: String,
        action_id: String,
        report_attempt: u32,
        final_status: String,
        execution_id: String,
        plan_digest: String,
        agent_id: String,
        instance_id: String,
        result_attestation: ResultAttestation,
        reported_at: String,
        result: ActionResultContract,
    ) -> Self {
        Self {
            api_version: API_VERSION_V1ALPHA1.to_string(),
            report_id,
            kind: REPORT_ACTION_RESULT_KIND.to_string(),
            dispatch_id: None,
            action_id,
            report_attempt,
            final_status,
            execution_id,
            plan_digest,
            agent_id,
            instance_id,
            result_attestation,
            reported_at,
            result,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultAttestation {
    pub result_digest: String,
    pub signature: String,
    pub issued_by: String,
    pub attested_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AckStatus {
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "queued")]
    Queued,
    #[serde(rename = "duplicate")]
    Duplicate,
    #[serde(rename = "stale")]
    Stale,
    #[serde(rename = "busy")]
    Busy,
}
