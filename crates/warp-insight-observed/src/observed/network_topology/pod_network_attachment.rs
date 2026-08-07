// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.NetworkTopology")]
pub struct PodNetworkAttachment {
    #[moju(unique)]
    pub attachment_id: String,
    pub pod_id: String,
    pub network_segment_id: String,
    pub interface_name: Option<String>,
    pub ip_addr: Option<String>,
    pub mac_addr: Option<String>,
    pub is_primary: bool,
    pub source: crate::AttachmentSource,
    pub valid_from: crate::DateTime,
    pub valid_to: Option<crate::DateTime>,
}
