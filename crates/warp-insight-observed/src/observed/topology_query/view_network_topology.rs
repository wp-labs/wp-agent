// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Observed", module = "Observed.TopologyQuery")]
pub struct ViewNetworkTopology {
    pub requested_by: String,
}
