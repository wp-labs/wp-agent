//! `ActionResult` contract types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResultContract {
    pub api_version: String,
    pub kind: String,
    pub action_id: String,
    pub execution_id: String,
    pub final_status: FinalStatus,
    pub step_records: Vec<StepActionRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalStatus {
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepActionRecord {
    pub step_id: String,
    pub status: StepStatus,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Succeeded,
    Failed,
    Skipped,
}
