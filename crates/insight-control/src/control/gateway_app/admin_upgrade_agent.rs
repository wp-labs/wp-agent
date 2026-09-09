// @jumo generated
// @jumo hash=3c0d307493621113

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Control", module = "Control.GatewayApp.UserFacingInterface")]
pub struct AdminUpgradeAgent {
    pub requested_by: String,
    pub agent_id: String,
    pub target_version: String,
}
