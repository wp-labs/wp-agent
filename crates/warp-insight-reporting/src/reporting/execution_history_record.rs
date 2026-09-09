// @jumo generated
// @jumo hash=29287a669303a0ed

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Pipeline")]
pub struct ExecutionHistoryRecord {
    pub plan_digest: String,
    pub action_id: String,
    pub final_status: String,
    pub finished_at: String,
    pub execution_id: String,
}
