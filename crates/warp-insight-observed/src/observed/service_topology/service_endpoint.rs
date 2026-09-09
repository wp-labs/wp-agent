// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Observed", module = "Observed.ServiceTopology")]
pub struct ServiceEndpoint {
    #[jumo(unique)]
    pub endpoint_id: String,
    pub service_id: String,
    pub endpoint_type: crate::ServiceEndpointType,
    pub address: String,
    pub port: Option<i64>,
    pub protocol: String,
    pub exposure_scope: crate::ExposureScope,
    pub valid_from: crate::DateTime,
    pub valid_to: Option<crate::DateTime>,
}
