// @jumo generated
// @jumo hash=a5b035a5a57b04aa

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "response", domain = "Control", module = "Control.GatewayApp.UserFacingInterface")]
pub struct AdminAgentInstallCodeReturned {
    pub install_code: crate::control::types::AgentInstallCode,
}
