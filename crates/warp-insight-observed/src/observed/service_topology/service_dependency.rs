// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.ServiceTopology")]
pub struct ServiceDependency {
    #[moju(unique)]
    pub dependency_id: String,
    pub upstream_service_id: String,
    pub downstream_service_id: String,
    pub dependency_type: crate::DependencyType,
    pub dependency_scope: crate::DependencyScope,
    pub criticality: Option<i64>,
    pub source: String,
    pub valid_from: crate::DateTime,
    pub valid_to: Option<crate::DateTime>,
}
