// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Observed", module = "Observed.NetworkTopology")]
pub struct NetworkSegment {
    #[jumo(unique)]
    pub network_segment_id: String,
    pub network_domain_id: String,
    pub segment_type: crate::NetworkSegmentType,
    pub name: String,
    pub cidr: Option<String>,
    pub gateway_ip: Option<String>,
}
