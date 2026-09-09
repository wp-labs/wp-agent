// @jumo generated
use super::{UpgradeStep, UpgradeTarget};

#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Control", module = "Control.InsightCenter.PlatformRelease")]
pub struct CreateUpgradePlan {
    pub targets: Vec<UpgradeTarget>,
    pub gateway_ids: Vec<String>,
    pub steps: Vec<UpgradeStep>,
    pub requested_by: String,
    pub requested_at: crate::DateTime,
}
