// @jumo generated
// @jumo hash=597f0206322a6683

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Pipeline")]
pub struct LocalReportInspection {
    pub has_result: String,
    pub execution_id: String,
    pub action_id: String,
    pub has_state: String,
    pub elapsed_since_finish_ms: String,
    pub has_runtime: String,
}
