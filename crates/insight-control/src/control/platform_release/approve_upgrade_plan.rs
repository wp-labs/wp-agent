// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.PlatformRelease")]
pub struct ApproveUpgradePlan {
    pub plan_id: String,
    pub approved_by: String,
    pub approved_at: crate::DateTime,
}
