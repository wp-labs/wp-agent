// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Observed", module = "Observed.ServiceTopology")]
pub struct BusinessDomain {
    #[jumo(unique)]
    pub business_id: String,
    pub tenant_id: String,
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
    pub lifecycle_state: Option<String>,
}
