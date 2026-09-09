// @jumo generated
// @jumo hash=b722159e25388a6a

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "event", domain = "Control", module = "Control.Agent.Identity")]
pub struct AgentCredentialIssued {
    pub agent_id: String,
    pub instance_id: String,
    pub credential: String,
    pub issued_at: String,
}
