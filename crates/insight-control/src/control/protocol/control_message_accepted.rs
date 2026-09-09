// @jumo generated
// @jumo hash=9a6d7bba16eb2752

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "event", domain = "Control", module = "Control.Protocol")]
pub struct ControlMessageAccepted {
    pub message_id: String,
    pub agent_id: String,
    pub instance_id: String,
    pub accepted_at: String,
}
