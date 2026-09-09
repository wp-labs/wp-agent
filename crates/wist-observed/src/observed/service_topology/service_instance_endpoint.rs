// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Observed", module = "Observed.ServiceTopology")]
pub struct ServiceInstanceEndpoint {
    #[jumo(unique)]
    pub instance_endpoint_id: String,
    pub service_instance_id: String,
    pub address: String,
    pub port: i64,
    pub protocol: String,
    pub source: String,
    pub valid_from: crate::DateTime,
    pub valid_to: Option<crate::DateTime>,
}
