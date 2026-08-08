// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.InsightCenter.PlatformRelease")]
pub struct PublishWarpAgentd {
    pub version: String,
    pub artifact_url: String,
    pub requested_by: String,
    pub requested_at: crate::DateTime,
}
