// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Control", module = "Control.InsightCenter.PlatformRelease")]
pub struct ApproveUpgradePlan {
    pub plan_id: String,
    pub approved_by: String,
    pub approved_at: crate::DateTime,
}
