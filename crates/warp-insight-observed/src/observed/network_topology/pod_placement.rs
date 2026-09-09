// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Observed", module = "Observed.NetworkTopology")]
pub struct PodPlacement {
    #[jumo(unique)]
    pub placement_id: String,
    pub pod_id: String,
    pub host_id: String,
    pub source: String,
    pub valid_from: crate::DateTime,
    pub valid_to: Option<crate::DateTime>,
}
