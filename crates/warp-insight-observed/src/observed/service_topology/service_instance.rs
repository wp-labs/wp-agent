// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Observed", module = "Observed.ServiceTopology")]
pub struct ServiceInstance {
    #[jumo(unique)]
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
