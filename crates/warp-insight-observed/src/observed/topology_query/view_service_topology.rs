// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Observed", module = "Observed.TopologyQuery")]
pub struct ViewServiceTopology {
    pub business_id: Option<String>,
    pub service_id: Option<String>,
    pub requested_by: String,
}
