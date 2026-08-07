// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.NetworkTopology")]
pub struct PodInventory {
    #[moju(unique)]
    pub pod_id: String,
    pub tenant_id: String,
    pub environment_id: String,
    pub cluster_id: Option<String>,
    pub namespace: String,
    pub workload_id: Option<String>,
    pub pod_uid: String,
    pub pod_name: String,
    pub node_id: Option<String>,
    pub phase: Option<String>,
    pub first_seen_at: crate::DateTime,
    pub last_seen_at: crate::DateTime,
}
