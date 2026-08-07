// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.HostInventory")]
pub struct HostDiskInventoryItem {
    pub device: Option<String>,
    pub mount_point: Option<String>,
    pub total_bytes: Option<i64>,
    pub filesystem: Option<String>,
}
