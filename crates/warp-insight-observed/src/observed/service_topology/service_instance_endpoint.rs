// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.ServiceTopology")]
pub struct ServiceInstanceEndpoint {
    #[moju(unique)]
    pub instance_endpoint_id: String,
    pub service_instance_id: String,
    pub address: String,
    pub port: i64,
    pub protocol: String,
    pub source: String,
    pub valid_from: crate::DateTime,
    pub valid_to: Option<crate::DateTime>,
}
