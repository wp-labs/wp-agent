// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.InsightCenter.PlatformRelease")]
pub struct WarpGateWayRelease {
    pub version: String,
    pub artifact_url: String,
    pub status: String,
    pub published_at: crate::DateTime,
}
