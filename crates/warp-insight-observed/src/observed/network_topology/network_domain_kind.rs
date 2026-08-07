// @moju generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "variant", domain = "Observed", module = "Observed.NetworkTopology")]
pub enum NetworkDomainKind {
    Vpc,
    VlanFabric,
    K8sClusterNetwork,
    CniFabric,
}
