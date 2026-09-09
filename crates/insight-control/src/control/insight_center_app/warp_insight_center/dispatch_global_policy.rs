// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Control", module = "Control.InsightCenterApp.WarpInsightCenter")]
pub struct DispatchGlobalPolicy {
    pub policy_version: String,
    pub gateway_ids: Vec<String>,
    pub requested_by: String,
}
