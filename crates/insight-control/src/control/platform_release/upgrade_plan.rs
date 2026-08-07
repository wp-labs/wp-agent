// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.PlatformRelease")]
pub struct UpgradePlan {
    pub plan_id: String,
    pub component: String,
    pub target_version: String,
    pub target_count: i64,
    pub status: String,
    pub created_at: crate::DateTime,
}
