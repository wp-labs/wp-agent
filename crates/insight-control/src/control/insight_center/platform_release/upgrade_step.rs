// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.InsightCenter.PlatformRelease")]
pub struct UpgradeStep {
    pub step_index: i64,
    pub gateway_ids: Vec<String>,
    pub status: String,
}
