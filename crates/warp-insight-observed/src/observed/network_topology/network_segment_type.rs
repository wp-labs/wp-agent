// @moju generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "variant", domain = "Observed", module = "Observed.NetworkTopology")]
pub enum NetworkSegmentType {
    Subnet,
    Overlay,
    PodNetwork,
    ServiceNetwork,
    NamespaceNetwork,
}
