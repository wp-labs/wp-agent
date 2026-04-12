//! `ActionPlan` contract types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPlanContract {
    pub api_version: String,
    pub kind: String,
    pub meta: ActionPlanMeta,
    pub target: ActionPlanTarget,
    pub constraints: ActionPlanConstraints,
    pub program: ActionPlanProgram,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPlanMeta {
    pub action_id: String,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPlanTarget {
    pub agent_id: String,
    pub instance_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPlanConstraints {
    pub max_total_duration_ms: u64,
    pub execution_profile: String,
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPlanProgram {
    pub entry: String,
    pub steps: Vec<ActionPlanStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPlanStep {
    pub id: String,
    pub op: String,
}
