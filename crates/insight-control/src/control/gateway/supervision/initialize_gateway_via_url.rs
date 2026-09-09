// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Control", module = "Control.Gateway.Supervision")]
pub struct InitializeGatewayViaUrl {
    pub init_url: String,
    pub requested_at: crate::DateTime,
}
