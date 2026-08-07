// @moju generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "variant", domain = "Observed", module = "Observed.HostResponsibility")]
pub enum HostGroupType {
    Application,
    Business,
    Cluster,
    Environment,
    OpsScope,
}
