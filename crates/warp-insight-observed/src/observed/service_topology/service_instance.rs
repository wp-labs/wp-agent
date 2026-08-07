// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.ServiceTopology")]
pub struct ServiceInstance {
    #[moju(unique)]
    pub service_instance_id: String,
    pub service_id: String,
    pub runtime_type: crate::ServiceRuntimeType,
    pub pod_id: Option<String>,
    pub process_id: Option<String>,
    pub host_id: Option<String>,
    pub version: Option<String>,
    pub state: Option<crate::ServiceInstanceLifecycleState>,
    pub started_at: Option<crate::DateTime>,
    pub last_seen_at: crate::DateTime,
}
