// @jumo generated
// @jumo hash=49d4bd67c642a1a3

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "event", domain = "Control", module = "Control.Agent.Enrollment")]
pub struct DuplicateRegistrationDetected {
    pub node_id: String,
    pub existing_agent_id: String,
    pub candidate_instance_id: String,
    pub action: String,
    pub detected_at: String,
}
