// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.HostInventory")]
pub struct HostInventory {
    #[moju(unique)]
    pub host_id: String,
    pub tenant_id: String,
    pub environment_id: String,
    pub host_name: String,
    pub machine_id: Option<String>,
    pub serial_number: Option<String>,
    pub cloud_instance_id: Option<String>,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub arch: String,
    pub os_name: String,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub cpu_model: Option<String>,
    pub cpu_core_count: Option<i64>,
    pub memory_total_bytes: Option<i64>,
    pub disk_inventory: Option<Vec<crate::HostDiskInventoryItem>>,
    pub network_interface_inventory: Option<Vec<crate::HostNetworkInterfaceInventoryItem>>,
    pub first_seen_at: crate::DateTime,
    pub last_inventory_at: crate::DateTime,
    pub inventory_revision: String,
}
