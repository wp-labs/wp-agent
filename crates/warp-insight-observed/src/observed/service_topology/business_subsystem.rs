// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.ServiceTopology")]
pub struct BusinessSubsystem {
    #[moju(unique)]
    pub subsystem_id: String,
    pub system_id: String,
    pub tenant_id: String,
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
}
