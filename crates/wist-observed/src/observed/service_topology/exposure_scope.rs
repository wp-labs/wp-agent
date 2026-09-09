// @jumo generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "variant", domain = "Observed", module = "Observed.ServiceTopology")]
pub enum ExposureScope {
    ClusterInternal,
    VpcInternal,
    Public,
}
