// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.InsightCenter.PlatformRelease")]
pub struct UpgradePlanApproval {
    pub plan_id: String,
    pub status: String,
    pub approved_by: String,
    pub approved_at: crate::DateTime,
}
