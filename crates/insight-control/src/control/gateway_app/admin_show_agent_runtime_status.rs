// @jumo generated
// @jumo hash=c55e816ea5aff35e

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Control", module = "Control.GatewayApp.UserFacingInterface")]
pub struct AdminShowAgentRuntimeStatus {
    pub requested_by: String,
    pub agent_id: String,
}
