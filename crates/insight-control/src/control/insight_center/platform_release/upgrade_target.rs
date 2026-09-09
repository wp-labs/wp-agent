// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.InsightCenter.PlatformRelease")]
pub struct UpgradeTarget {
    pub component: String,
    pub target_version: String,
}
