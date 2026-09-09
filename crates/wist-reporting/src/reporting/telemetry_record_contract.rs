// @jumo generated
// @jumo hash=be7967fcf278dc28

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Contract")]
pub struct TelemetryRecordContract {
    pub kind: String,
    #[jumo(unique)]
    pub record_id: String,
    pub agent_id: String,
    pub instance_id: String,
    pub collected_at: String,
    pub api_version: String,
}
