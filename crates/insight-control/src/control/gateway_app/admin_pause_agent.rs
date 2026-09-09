// @jumo generated
// @jumo hash=3713107a9dcda46f

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Control", module = "Control.GatewayApp.UserFacingInterface")]
pub struct AdminPauseAgent {
    pub agent_id: String,
    pub requested_by: String,
}
