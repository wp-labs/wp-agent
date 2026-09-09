// @jumo generated
// @jumo hash=ea09717ff74ceafa

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "event", domain = "Control", module = "Control.Protocol")]
pub struct ControlMessageRejected {
    pub message_id: String,
    pub agent_id: String,
    pub instance_id: String,
    pub reason_code: String,
    pub rejected_at: String,
}
