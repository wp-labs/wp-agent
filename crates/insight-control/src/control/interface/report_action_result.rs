// @jumo generated
// @jumo hash=d1bd562097f2181c

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Reporting", module = "Reporting.Protocol")]
pub struct ReportActionResult {
    pub execution_id: String,
    pub kind: String,
    pub agent_id: String,
    pub result_attestation: wist_reporting::ResultAttestation,
    pub action_id: String,
    pub reported_at: crate::control::types::DateTime,
    pub final_status: String,
    pub result: String,
    pub dispatch_id: String,
    pub plan_digest: String,
    pub report_attempt: i64,
    pub report_id: String,
    pub api_version: String,
    pub instance_id: String,
}
