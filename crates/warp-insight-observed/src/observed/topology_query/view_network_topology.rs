// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Observed", module = "Observed.TopologyQuery")]
pub struct ViewNetworkTopology {
    pub requested_by: String,
}
