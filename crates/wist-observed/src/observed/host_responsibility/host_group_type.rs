// @jumo generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "variant", domain = "Observed", module = "Observed.HostResponsibility")]
pub enum HostGroupType {
    Application,
    Business,
    Cluster,
    Environment,
    OpsScope,
}
