// @jumo generated
// @jumo hash=c1dd2de3c508b773

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Pipeline")]
pub struct ReportingRequest {
    pub instance_id: String,
    pub agent_id: String,
    pub state_dir: String,
    pub execution_id: String,
}
