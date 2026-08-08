// @moju generated
// @moju hash=f89352363bbea22d

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.AgentApp.FacingInterface")]
pub struct SubmitEnrollmentRequest {
    pub capability_summary: String,
    pub token: String,
    pub requested_at: crate::control::types::DateTime,
    pub host_profile: crate::control::types::AgentHostProfile,
    pub credential_request: String,
}
