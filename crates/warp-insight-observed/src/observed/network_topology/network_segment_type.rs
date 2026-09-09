// @jumo generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "variant", domain = "Observed", module = "Observed.NetworkTopology")]
pub enum NetworkSegmentType {
    Subnet,
    Overlay,
    PodNetwork,
    ServiceNetwork,
    NamespaceNetwork,
}
