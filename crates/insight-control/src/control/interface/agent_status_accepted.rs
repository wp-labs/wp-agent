// @jumo generated
// @jumo hash=ed0b445234fa1dee

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "response", domain = "Reporting", module = "Reporting.Protocol")]
pub struct AgentStatusAccepted {
    pub snapshot: wist_reporting::RuntimeHealthSnapshot,
}
