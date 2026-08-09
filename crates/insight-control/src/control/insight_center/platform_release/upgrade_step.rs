// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.InsightCenter.PlatformRelease")]
pub struct UpgradeStep {
    pub step_index: i64,
    pub gateway_ids: Vec<String>,
    pub status: String,
}
