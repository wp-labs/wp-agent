// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Gateway.Supervision")]
pub struct GatewayStatusAccepted {
    pub gateway_id: String,
    pub instance_id: String,
    pub accepted_at: crate::DateTime,
}
