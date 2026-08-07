// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.ServiceTopology")]
pub struct ServiceEntity {
    #[moju(unique)]
    pub service_id: String,
    pub tenant_id: String,
    pub business_id: Option<String>,
    pub system_id: Option<String>,
    pub subsystem_id: Option<String>,
    pub namespace: Option<String>,
    pub name: String,
    pub service_type: crate::ServiceType,
    pub language: Option<String>,
    pub lifecycle_state: Option<String>,
}
