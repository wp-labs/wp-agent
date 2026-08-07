// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.WarpInsightCenter")]
pub struct DispatchGlobalPolicy {
    pub policy_version: String,
    pub gateway_ids: Vec<String>,
    pub requested_by: String,
}
