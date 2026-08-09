// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.InsightCenter.PlatformRelease")]
pub struct UpgradeTarget {
    pub component: String,
    pub target_version: String,
}
