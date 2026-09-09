// @jumo generated
use super::{UpgradeStep, UpgradeTarget};

#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.InsightCenter.PlatformRelease")]
pub struct UpgradePlan {
    pub plan_id: String,
    pub targets: Vec<UpgradeTarget>,
    pub target_count: i64,
    pub status: String,
    pub created_at: crate::DateTime,
    pub steps: Vec<UpgradeStep>,
}
