// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.PlatformRelease")]
pub struct CreateUpgradePlan {
    pub component: String,
    pub target_version: String,
    pub gateway_ids: Vec<String>,
    pub requested_by: String,
    pub requested_at: crate::DateTime,
}
