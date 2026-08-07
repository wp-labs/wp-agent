// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.PlatformRelease")]
pub struct UpgradePlanApproval {
    pub plan_id: String,
    pub status: String,
    pub approved_by: String,
    pub approved_at: crate::DateTime,
}
