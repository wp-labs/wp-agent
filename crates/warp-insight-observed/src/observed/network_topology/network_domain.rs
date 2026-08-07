// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.NetworkTopology")]
pub struct NetworkDomain {
    #[moju(unique)]
    pub network_domain_id: String,
    pub tenant_id: String,
    pub environment_id: String,
    pub kind: crate::NetworkDomainKind,
    pub name: String,
    pub external_ref: Option<String>,
}
