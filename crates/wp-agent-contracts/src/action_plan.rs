//! `ActionPlan` contract types.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::API_VERSION_V1;

pub const ACTION_PLAN_KIND: &str = "action_plan";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPlanContract {
    pub api_version: String,
    pub kind: String,
    pub meta: ActionPlanMeta,
    pub target: ActionPlanTarget,
    pub constraints: ActionPlanConstraints,
    pub program: ActionPlanProgram,
}

impl ActionPlanContract {
    pub fn new(
        meta: ActionPlanMeta,
        target: ActionPlanTarget,
        constraints: ActionPlanConstraints,
        program: ActionPlanProgram,
    ) -> Self {
        Self {
            api_version: API_VERSION_V1.to_string(),
            kind: ACTION_PLAN_KIND.to_string(),
            meta,
            target,
            constraints,
            program,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPlanMeta {
    pub action_id: String,
    pub request_id: String,
    pub template_id: Option<String>,
    pub tenant_id: String,
    pub environment_id: String,
    pub plan_version: u64,
    pub compiled_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPlanTarget {
    pub agent_id: String,
    pub instance_id: Option<String>,
    pub node_id: String,
    pub host_name: Option<String>,
    pub platform: String,
    pub arch: String,
    #[serde(default)]
    pub selectors: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPlanConstraints {
    pub risk_level: RiskLevel,
    pub approval_ref: Option<String>,
    pub approval_mode: ApprovalMode,
    pub requested_by: String,
    pub reason: Option<String>,
    pub max_total_duration_ms: u64,
    pub step_timeout_default_ms: u64,
    pub execution_profile: String,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    #[serde(rename = "R0")]
    R0,
    #[serde(rename = "R1")]
    R1,
    #[serde(rename = "R2")]
    R2,
    #[serde(rename = "R3")]
    R3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalMode {
    #[serde(rename = "not_required")]
    NotRequired,
    #[serde(rename = "required")]
    Required,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPlanProgram {
    pub entry: String,
    pub steps: Vec<ActionPlanStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPlanStep {
    pub id: String,
    pub kind: String,
    pub op: Option<String>,
}
